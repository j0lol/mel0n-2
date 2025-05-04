use bevy::{math::bounding::IntersectsVolume, prelude::*};
use helpers::bounding_circle;

use crate::{
    Gravity, Velocity,
    fruit::{Collided, Diameter, Fruit},
};

const ELASTICITY: f32 = 0.7;
const MASS: f32 = 16.0;
const INV_MASS: f32 = 1. / MASS;

#[derive(Event)]
pub struct CollisionEvent();

#[derive(Copy, Clone)]
pub struct Body {
    restitution: f32,
    velocity: Vec2,
    inverse_mass: f32, // 1 / mass (1/1=1)
}
pub struct Contact {
    normal: Vec2,
    a: Body,
    b: Body,
}

// Linear Collision Resolution
// https://youtu.be/1L2g4ZqmFLQ
fn resolve_collision(contact: Contact) -> Vec2 {
    use log::info;

    assert!(
        contact.normal.is_normalized(),
        "A normalized normal vector is needed."
    );
    let a = contact.a;
    let b = contact.b;

    // Elasticity (coefficient of restitution)
    let e = f32::min(a.restitution, b.restitution);
    assert!(e <= 1.0);
    assert!(e >= 0.0);

    let rel_v = a.velocity - b.velocity;
    info!("rel_v {rel_v}");

    let impulse_mag = -(1. + e) * rel_v.dot(contact.normal) / (a.inverse_mass + b.inverse_mass);
    info!("J {impulse_mag}");
    let impulse_dir = contact.normal;

    // assert!(
    //     impulse_mag < 80.0,
    //     "Ur physics is fucked: mag {impulse_mag} \n			= -(1. + {e}) * {rel_v}.dot({}) / ({} + {}) \n			= {} / {}",
    //     contact.normal,
    //     a.inverse_mass,
    //     b.inverse_mass,
    //     -(1. + e) * rel_v.dot(contact.normal),
    //     (a.inverse_mass + b.inverse_mass)
    // );
    impulse_dir * impulse_mag
}

#[derive(Component, Default, Debug)]
pub struct Physics;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

pub fn apply_gravity(mut entities: Query<&mut Velocity, With<Gravity>>) {
    const TERMINAL_VELOCITY: f32 = 20.0;
    for mut velocity in &mut entities {
        velocity.0.y = (velocity.0.y + 0.4).clamp(-TERMINAL_VELOCITY, TERMINAL_VELOCITY);
    }
}

// Air and ground "friction"
pub fn apply_friction(mut entities: Query<&mut Velocity>) {
    const FRICTION_COEFFICIENT: f32 = 0.95;
    for mut velocity in &mut entities {
        velocity.0.x *= FRICTION_COEFFICIENT;
        velocity.0.y *= FRICTION_COEFFICIENT;
    }
}

pub fn apply_velocity(mut entities: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut entities {
        transform.translation.x += velocity.0.x;
        transform.translation.y += velocity.0.y;
    }
}

pub fn apply_collisions(
    mut query: Query<
        (&mut Transform, &Diameter, &mut Velocity, &mut Collided),
        (With<Physics>, With<Fruit>),
    >,
) {
    let mut combinations = query.iter_combinations_mut();
    while let Some(
        [
            (mut a_trans, a_diam, mut a_vel, mut a_coltimes),
            (mut b_trans, b_diam, mut b_vel, mut b_coltimes),
        ],
    ) = combinations.fetch_next()
    {
        let a_bound = bounding_circle(*a_diam, a_trans.translation);
        let b_bound = bounding_circle(*b_diam, b_trans.translation);

        if !a_bound.intersects(&b_bound) {
            continue;
        }

        a_coltimes.0 += 1;
        b_coltimes.0 += 1;

        log::info!("bop!");

        let normal_dir = a_trans.translation.xy().angle_to(b_trans.translation.xy());
        let normal = Vec2::from_angle(normal_dir).normalize();

        log::info!("NORM {normal} ({normal_dir})");

        let impulse = resolve_collision(Contact {
            normal,
            a: Body {
                restitution: ELASTICITY,
                velocity: a_vel.0,
                inverse_mass: INV_MASS,
            },
            b: Body {
                restitution: ELASTICITY,
                velocity: b_vel.0,
                inverse_mass: INV_MASS,
            },
        });

        log::info!("jv {impulse}");

        log::info!(
            "Obj A changed speed by {}x",
            a_vel.0.length() / impulse.length()
        );
        log::info!(
            "Obj B changed speed by {}x",
            b_vel.0.length() / -impulse.length()
        );

        log::info!("Rel V before: {}", a_vel.0.length() - b_vel.0.length());
        log::info!("Rel V after: {}", impulse.length() - -impulse.length());

        a_vel.0 += impulse;
        b_vel.0 -= impulse;
    }
}

pub mod helpers {
    use bevy::{
        math::bounding::{Aabb2d, BoundingCircle},
        prelude::*,
    };

    use crate::{Collider, Vec3, fruit::Diameter};

    pub fn bounding_circle(diameter: Diameter, translation: Vec3) -> BoundingCircle {
        let radius = diameter.0 / 2.;
        BoundingCircle::new(translation.truncate() + Vec2::splat(radius), radius)
    }

    pub fn aabb2d(translation: Vec3, collider: &Collider) -> Aabb2d {
        Aabb2d::new(
            translation.truncate() + collider.half_size,
            collider.half_size,
        )
    }
}

#[cfg(test)]
mod test {

    use core::error::Error;

    use assert_float_eq::assert_float_relative_eq;
    use bevy::{
        log::info,
        math::{Vec2, ops::abs, vec2},
        prelude::Box,
        reflect::impl_reflect_opaque,
    };

    use super::{Body, Contact, resolve_collision};

    #[test]
    pub fn conservation_of_energy() {
        let mut a = Body {
            restitution: 1.0,
            velocity: vec2(20.0, 0.),
            inverse_mass: 1.0,
        };
        let mut b = Body {
            restitution: 1.0,
            velocity: vec2(-20.0, 0.),
            inverse_mass: 1.0,
        };
        let contact = Contact {
            normal: Vec2::X,
            a,
            b,
        };

        let lhs = (a.velocity - b.velocity).length();

        let impulse = resolve_collision(contact);
        a.velocity += impulse;
        b.velocity -= impulse;

        let rhs = (a.velocity - b.velocity).length();

        assert_float_relative_eq!(a.velocity.x, -20.);
        assert_float_relative_eq!(b.velocity.x, 20.);

        // Weak comparison of floats
        assert_float_relative_eq!(lhs, rhs);
    }
    #[test]
    pub fn inelastic_collision_standstill() {
        let mut a = Body {
            restitution: 0.0,
            velocity: vec2(20.0, 0.),
            inverse_mass: 1.0,
        };
        let mut b = Body {
            restitution: 0.0,
            velocity: vec2(-20.0, 0.),
            inverse_mass: 1.0,
        };
        let contact = Contact {
            normal: Vec2::X,
            a,
            b,
        };

        let impulse = resolve_collision(contact);
        a.velocity += impulse;
        b.velocity -= impulse;

        assert_float_relative_eq!(a.velocity.length(), 0.0);
        assert_float_relative_eq!(b.velocity.length(), 0.0);
    }
}
