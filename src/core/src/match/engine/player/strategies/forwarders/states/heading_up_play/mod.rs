use crate::common::loader::DefaultNeuralNetworkLoader;
use crate::common::NeuralNetwork;
use crate::r#match::forwarders::states::ForwardState;
use crate::r#match::player::events::PlayerEvent;
use crate::r#match::position::VectorExtensions;
use crate::r#match::{
    ConditionContext, MatchPlayer, PlayerSide, StateChangeResult, StateProcessingContext,
    StateProcessingHandler,
};
use nalgebra::Vector3;
use std::sync::LazyLock;

static FORWARD_HEADING_UP_PLAY_STATE_NETWORK: LazyLock<NeuralNetwork> = LazyLock::new(|| {
    DefaultNeuralNetworkLoader::load(include_str!("nn_heading_up_play_data.json"))
});

#[derive(Default)]
pub struct ForwardHeadingUpPlayState {}

impl StateProcessingHandler for ForwardHeadingUpPlayState {
    fn try_fast(&self, ctx: &StateProcessingContext) -> Option<StateChangeResult> {
        let mut result = StateChangeResult::new();

        let player_ops = ctx.player();

        // Check if the player has the ball
        if !ctx.player.has_ball {
            // Transition to Running state if the player doesn't have the ball
            return Some(StateChangeResult::with_forward_state(ForwardState::Running));
        }

        // Check if the player is under pressure
        if player_ops.is_under_pressure() {
            // Transition to Passing state if under pressure
            return Some(StateChangeResult::with_forward_state(ForwardState::Passing));
        }

        // Check if there's support from teammates
        if !self.has_support(ctx) {
            // Transition to Dribbling state if there's no support
            return Some(StateChangeResult::with_forward_state(
                ForwardState::Dribbling,
            ));
        }

        // Check if there's an opportunity to pass to a teammate
        if let Some(teammate_id) = self.find_best_pass_option(ctx) {
            let teammate = &ctx.context.players.get(teammate_id)?;

            // Perform the pass
            result
                .events
                .add_player_event(PlayerEvent::RequestPass(ctx.player.id));

            // Transition to Running state after making the pass
            return Some(StateChangeResult::with_forward_state(ForwardState::Running));
        }

        // Move towards the opponent's goal
        let goal_position = ctx.ball().direction_to_opponent_goal();
        let direction = (goal_position - ctx.player.position).normalize();
        result.velocity = Some(direction * ctx.player.skills.physical.acceleration * 0.5);

        Some(result)
    }

    fn process_slow(&self, ctx: &StateProcessingContext) -> Option<StateChangeResult> {
        None
    }

    fn velocity(&self, ctx: &StateProcessingContext) -> Option<Vector3<f32>> {
        Some(Vector3::new(0.0, 0.0, 0.0))
    }

    fn process_conditions(&self, ctx: ConditionContext) {}
}

impl ForwardHeadingUpPlayState {
    fn has_support(&self, ctx: &StateProcessingContext) -> bool {
        let teammates = ctx.context.players.get_by_team(ctx.player.team_id);
        let min_support_distance = 10.0; // Adjust based on your game's scale

        teammates.iter().any(|teammate| {
            let distance = ctx.player.position.distance_to(&teammate.position);
            distance < min_support_distance
        })
    }

    fn find_best_pass_option(&self, ctx: &StateProcessingContext) -> Option<u32> {
        let teammates = ctx.context.players.get_by_team(ctx.player.team_id);

        teammates
            .iter()
            .enumerate()
            .filter(|(_, teammate)| {
                // Check if the teammate is in a good position to receive a pass
                let is_open = self.is_open_for_pass(ctx, teammate);
                let is_in_passing_lane = self.in_passing_lane(ctx, teammate);
                is_open && is_in_passing_lane
            })
            .max_by(|(_, a), (_, b)| {
                // Find the teammate with the highest scoring chance
                let score_a = self.scoring_chance(ctx, a);
                let score_b = self.scoring_chance(ctx, b);
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(index, player)| player.id)
    }

    fn is_open_for_pass(&self, ctx: &StateProcessingContext, teammate: &MatchPlayer) -> bool {
        let max_distance = 20.0; // Adjust based on your game's scale

        let players = ctx.team();
        let opponents = players.opponents();

        // Check if the teammate is within a reasonable distance
        if ctx.player.position.distance_to(&teammate.position) > max_distance {
            return false;
        }

        // Check if there are no opponents close to the teammate
        opponents
            .iter()
            .all(|opponent| opponent.position.distance_to(&teammate.position) > 5.0)
    }

    fn in_passing_lane(&self, ctx: &StateProcessingContext, teammate: &MatchPlayer) -> bool {
        let ball_position = ctx.tick_context.object_positions.ball_position;
        let player_to_ball = (ball_position - ctx.player.position).normalize();
        let player_to_teammate = (teammate.position - ctx.player.position).normalize();

        // Check if the teammate is in the passing lane
        player_to_ball.dot(&player_to_teammate) > 0.8
    }

    fn scoring_chance(&self, ctx: &StateProcessingContext, teammate: &MatchPlayer) -> f32 {
        let goal_position = match teammate.side {
            Some(PlayerSide::Left) => ctx.context.goal_positions.right,
            Some(PlayerSide::Right) => ctx.context.goal_positions.left,
            _ => Vector3::new(0.0, 0.0, 0.0),
        };

        let distance_to_goal = teammate.position.distance_to(&goal_position);

        // Calculate the scoring chance based on distance to the goal
        1.0 - distance_to_goal / ctx.context.field_size.width as f32
    }
}
