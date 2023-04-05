﻿use crate::r#match::position::VectorExtensions;
use crate::r#match::MatchPlayer;
use nalgebra::Vector3;

pub enum SteeringBehavior<'p> {
    Seek {
        target: Vector3<f32>,
    },
    Arrive {
        target: Vector3<f32>,
        slowing_distance: f32,
    },
    Pursuit {
        target: &'p MatchPlayer,
    },
    Evade {
        target: &'p MatchPlayer,
    },
    Wander {
        target: Vector3<f32>,
        radius: f32,
        jitter: f32,
        distance: f32,
        angle: f32,
    },
    Flee {
        target: Vector3<f32>,
    },
}

impl<'p> SteeringBehavior<'p> {
    pub fn calculate(&self, player: &MatchPlayer) -> SteeringOutput {
        match self {
            SteeringBehavior::Seek { target } => {
                let desired_velocity = (*target - player.position).normalize();
                let steering = desired_velocity - player.velocity;
                SteeringOutput {
                    velocity: steering,
                    rotation: 0.0,
                }
            }
            SteeringBehavior::Arrive {
                target,
                slowing_distance,
            } => {
                let distance = (*target - player.position).length();
                if distance < *slowing_distance {
                    let desired_speed = distance / *slowing_distance * player.skills.max_speed();
                    let desired_velocity = (*target - player.position).normalize() * desired_speed;
                    let steering = desired_velocity - player.velocity;
                    SteeringOutput {
                        velocity: steering,
                        rotation: 0.0,
                    }
                } else {
                    let desired_velocity =
                        (*target - player.position).normalize() * player.skills.max_speed();
                    let steering = desired_velocity - player.velocity;
                    SteeringOutput {
                        velocity: steering,
                        rotation: 0.0,
                    }
                }
            }
            SteeringBehavior::Pursuit { target } => {
                let distance = (target.position - player.position).length();
                let prediction = distance / player.skills.max_speed();
                let target_position = target.position + (target.velocity * prediction);
                let desired_velocity =
                    (target_position - player.position).normalize() * player.skills.max_speed();
                let steering = desired_velocity - player.velocity;
                SteeringOutput {
                    velocity: steering,
                    rotation: 0.0,
                }
            }
            SteeringBehavior::Evade { target } => {
                let distance = (target.position - player.position).length();
                let prediction = distance / player.skills.max_speed();
                let target_position = target.position + target.velocity * prediction;
                let desired_velocity =
                    (player.position - target_position).normalize() * player.skills.max_speed();
                let steering = desired_velocity - player.velocity;
                SteeringOutput {
                    velocity: steering,
                    rotation: 0.0,
                }
            }
            SteeringBehavior::Wander {
                target,
                radius,
                jitter,
                distance,
                angle,
            } => {
                let rand_vec = Vector3::random_in_unit_circle() * *jitter;
                let target = rand_vec + *target;
                let target_offset = target - player.position;
                let mut target_offset = target_offset.normalize() * *distance;
                target_offset = target_offset.add_scalar(player.heading() * *radius);
                let steering = target_offset - player.velocity;
                SteeringOutput {
                    velocity: steering,
                    rotation: 0.0,
                }
            }
            SteeringBehavior::Flee { target } => {
                let desired_velocity =
                    (player.position - *target).normalize() * player.skills.max_speed();
                let steering = desired_velocity - player.velocity;
                SteeringOutput {
                    velocity: steering,
                    rotation: 0.0,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SteeringOutput {
    pub velocity: Vector3<f32>,
    pub rotation: f32,
}

impl SteeringOutput {
    pub fn new(velocity: Vector3<f32>, rotation: f32) -> Self {
        SteeringOutput { velocity, rotation }
    }
}
