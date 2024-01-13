use bitmask_enum::bitmask;
use glam::{Quat, Vec3};
use nalgebra::Vector3;
use once_cell::sync::Lazy;
use rapier3d::{
    dynamics::{RigidBodySet, IntegrationParameters, IslandManager, ImpulseJointSet, MultibodyJointSet, CCDSolver, RigidBody, RigidBodyBuilder, RigidBodyHandle},
    geometry::{ColliderSet, BroadPhase, NarrowPhase, ColliderBuilder, ColliderHandle, InteractionGroups, Ray}, 
    na::vector,
    pipeline::{PhysicsPipeline, QueryPipeline, QueryFilter},
    math::{Point, Real}
};
use crate::{objects::Transform, math_utils::{rad_vec_to_deg, deg_to_rad}};
use super::debugger;


const GRAVITY: Vector3<f32> = vector![0.0, -9.81, 0.0];
pub static mut RIGID_BODY_SET: Lazy<RigidBodySet> = Lazy::new(|| RigidBodySet::new());
pub static mut COLLIDER_SET: Lazy<ColliderSet> = Lazy::new(|| ColliderSet::new());
static INTEGRATION_PARAMETERS: Lazy<IntegrationParameters> = Lazy::new(|| IntegrationParameters::default());
static mut PHYSICS_PIPELINE: Lazy<PhysicsPipeline> = Lazy::new(|| PhysicsPipeline::new());
static mut ISLAND_MANAGER: Lazy<IslandManager> = Lazy::new(|| IslandManager::new());
static mut BROAD_PHASE: Lazy<BroadPhase> = Lazy::new(|| BroadPhase::new());
pub static mut NARROW_PHASE: Lazy<NarrowPhase> = Lazy::new(|| NarrowPhase::new());
static mut IMPULSE_JOINT_SET: Lazy<ImpulseJointSet> = Lazy::new(|| ImpulseJointSet::new());
static mut MULTIBODY_JOINT_SET: Lazy<MultibodyJointSet> = Lazy::new(|| MultibodyJointSet::new());
static mut CCD_SOLVER: Lazy<CCDSolver> = Lazy::new(|| CCDSolver::new());
static mut QUERY_PIPELINE: Lazy<QueryPipeline> = Lazy::new(|| QueryPipeline::new());


pub fn update() {
    unsafe {
        PHYSICS_PIPELINE.step(
            &GRAVITY, 
            &INTEGRATION_PARAMETERS, 
            &mut ISLAND_MANAGER, 
            &mut BROAD_PHASE,
            &mut NARROW_PHASE,
            &mut RIGID_BODY_SET,
            &mut COLLIDER_SET,
            &mut IMPULSE_JOINT_SET, 
            &mut MULTIBODY_JOINT_SET,
            &mut CCD_SOLVER,
            Some(&mut QUERY_PIPELINE),
            &(), &());
        QUERY_PIPELINE.update(&mut RIGID_BODY_SET, &mut COLLIDER_SET);
    }
}

pub fn remove_rigid_body(body_parameters: &mut ObjectBodyParameters) {
    if let Some(handle) = body_parameters.rigid_body_handle {
        unsafe {
            RIGID_BODY_SET.remove(handle, &mut ISLAND_MANAGER, &mut COLLIDER_SET, &mut IMPULSE_JOINT_SET, &mut MULTIBODY_JOINT_SET, true);
        }
        body_parameters.rigid_body_handle = None;
    }
}

pub fn new_rigid_body(body_type: BodyType, transform: Option<Transform>, mass: f32, id: u128,
    membership_groups: Option<CollisionGroups>, filter_groups: Option<CollisionGroups>) -> ObjectBodyParameters {
    let mut collider_option: Option<BodyColliderType> = None;
    let rigid_body_builder = match body_type {
        BodyType::Fixed(collider) => {
            collider_option = collider;
            RigidBodyBuilder::fixed()
        },
        BodyType::Dynamic(collider) => {
            collider_option = collider;
            RigidBodyBuilder::dynamic()
        },
        BodyType::VelocityKinematic(collider) => {
            collider_option = collider;
            RigidBodyBuilder::kinematic_velocity_based()
        },
        BodyType::PositionKinematic(collider) => {
            collider_option = collider;
            RigidBodyBuilder::kinematic_position_based()
        },
    };

    let rigid_body: RigidBody;

    match transform {
        Some(transform) => {
            rigid_body = rigid_body_builder
                .additional_mass(mass)
                .translation(transform.position.into())
                .rotation(transform.rotation.into())
                .user_data(id)
                .build();
        },
        None => {
            rigid_body = rigid_body_builder
                .additional_mass(mass)
                .user_data(id)
                .build();
        },
    }

    let collider_builder: ColliderBuilder;

    match collider_option {
        Some(collider) => {
            let membership_groups = match membership_groups {
                Some(groups) => groups,
                None => CollisionGroups::Group1,
            };

            let filter_groups = match filter_groups {
                Some(groups) => groups,
                None => CollisionGroups::Group1,
            };

            collider_builder = collider_type_to_collider_builder(collider, membership_groups, filter_groups);
        },
        None => {
            unsafe {
                let rigid_body_handle = RIGID_BODY_SET.insert(rigid_body);
                let body_parameters = ObjectBodyParameters {
                    rigid_body_handle: Some(rigid_body_handle),
                    collider_handle: None,
                    render_collider_type: None,
                };

                return body_parameters;
            }
        },
    }

    let collider = collider_builder.build();

    unsafe {
        let rigid_body_handle = RIGID_BODY_SET.insert(rigid_body);
        let collider_handle = COLLIDER_SET.insert_with_parent(collider, rigid_body_handle, &mut RIGID_BODY_SET);
        let body_parameters = ObjectBodyParameters {
            rigid_body_handle: Some(rigid_body_handle),
            collider_handle: Some(collider_handle),
            render_collider_type: None
        };

        return body_parameters;
    }
}


#[derive(Debug, Clone, Copy)]
pub struct ObjectBodyParameters {
    pub rigid_body_handle: Option<RigidBodyHandle>,
    pub collider_handle: Option<ColliderHandle>,
    pub render_collider_type: Option<RenderColliderType>
}

impl ObjectBodyParameters {
    pub fn empty() -> ObjectBodyParameters {
        ObjectBodyParameters { rigid_body_handle: None, collider_handle: None, render_collider_type: None }
    }
    /*
    pub fn get_rigid_body_handle(&self) -> Option<&RigidBodyHandle> {
        self.rigid_body_handle.as_ref()
    }

    pub fn get_rigid_body_handle_mut(&mut self) -> Option<&mut RigidBodyHandle> {
        self.rigid_body_handle.as_mut()
    }

    pub fn get_collider_handle(&self) -> &Option<ColliderHandle> {
        &self.collider_handle
    }

    pub fn get_collider_handle_mut(&mut self) -> &mut Option<ColliderHandle> {
        &mut self.collider_handle
    }

    pub fn set_mass(&mut self, mass: f32) {
        if let Some(body) = &self.rigid_body_handle {
            match get_body(body) {
                Some(body) => body.set_additional_mass(mass, true),
                None => debugger::error(&format!("set_mass error!\nfailed to get rigid body with handle {:?}", self.rigid_body_handle)),
            }
        } else {
            debugger::error("set_mass error!\nrigid body handle is None");
        }
    }

    pub fn get_mass(&self) -> Option<f32> {
        if let Some(body) = &self.rigid_body_handle {
            match get_body(body) {
                Some(body) => Some(body.mass().into()),
                None => {
                    debugger::error(&format!("get_mass error!\nfailed to get rigid body with handle {:?}", self.rigid_body_handle));
                    None
                },
            }
        } else {
            debugger::error("get_mass error!\nrigid body handle is None");
            None
        }
    }

    pub fn set_collider(&mut self, collider_type: BodyColliderType) {
        if let Some(body) = &self.rigid_body_handle {
            match get_body(body) {
                Some(_) => {
                    match self.collider_handle {
                        Some(collider) => {
                            unsafe {
                                COLLIDER_SET.remove(collider, &mut ISLAND_MANAGER, &mut RIGID_BODY_SET, true);
                                let new_collider = collider_type_to_collider_builder(collider_type).build();

                                self.collider_handle = 
                                    Some(COLLIDER_SET.insert_with_parent(new_collider, *body, &mut RIGID_BODY_SET));
                            }
                        },
                        None => (),
                    }
                },
                None => debugger::error(&format!("set_collider error!\nfailed to get rigid body with handle {:?}", self.rigid_body_handle)),
            }
        } else {
            debugger::error("set_collider error!\nrigid body handle is None");
        }
    }

    pub fn set_transform(&mut self, transform: Transform) {
        if let Some(body) = &self.rigid_body_handle {
            let body = get_body(body);

            match body {
                Some(body) => {
                    body.set_position(transform.position.into(), false);
                    let rad_rot = deg_vec_to_rad(transform.rotation);
                    body.set_rotation(Quat::from_euler(glam::EulerRot::XYZ, rad_rot.x, rad_rot.y, rad_rot.z).into(), false);
                },
                None => debugger::error(&format!("set_transform error!\nfailed to get rigid body with handle {:?}", self.rigid_body_handle)),
            }
        } else {
            debugger::error("set_transform error!\nrigid body handle is None");
        }
    }

    pub fn get_collider(&self) -> Option<RenderColliderType> {
        self.render_collider_type
    }*/

    pub fn set_render_collider(&mut self, render_collider: Option<RenderColliderType>) { 
        self.render_collider_type = render_collider;
    }
}


fn get_body(handle: &RigidBodyHandle) -> Option<&mut RigidBody> {
    unsafe {
        RIGID_BODY_SET.get_mut(*handle)
    }
}

pub fn collider_type_to_collider_builder(collider: BodyColliderType, membership_groups: CollisionGroups, filter_groups: CollisionGroups) -> ColliderBuilder {
    let mut collider_builder: ColliderBuilder;

    match collider {
        BodyColliderType::Ball(radius) => collider_builder = ColliderBuilder::ball(radius),
        BodyColliderType::Cuboid(x, y, z) => collider_builder = ColliderBuilder::cuboid(x, y, z),
        BodyColliderType::Capsule(radius, height) => collider_builder = ColliderBuilder::capsule_y(height / 2.0, radius),
        BodyColliderType::Cylinder(radius, height) => collider_builder = ColliderBuilder::cylinder(height / 2.0, radius),
        BodyColliderType::TriangleMesh(verts_positions, indices) => {
            let mut positions_nalgebra: Vec<Point<Real>> = Vec::new();
            verts_positions.iter().for_each(|pos| positions_nalgebra.push((*pos).into()));
            collider_builder = ColliderBuilder::trimesh(positions_nalgebra, indices.into());
        },
    }

    collider_builder = collider_builder.collision_groups(InteractionGroups::new(membership_groups.bits.into(), filter_groups.bits.into()));

    collider_builder
}


pub enum BodyType {
    Fixed(Option<BodyColliderType>),
    Dynamic(Option<BodyColliderType>), 
    VelocityKinematic(Option<BodyColliderType>), 
    PositionKinematic(Option<BodyColliderType>), 
}

pub enum BodyColliderType {
    /// f32 is radius,
    Ball(f32),
    /// f32s are half-x, half-y and half-z size of collider,
    Cuboid(f32, f32, f32),
    /// first is radius, second is height,
    Capsule(f32, f32),
    /// first is radius, second is height,
    Cylinder(f32, f32),
    /// first is verts position, second is indices,
    TriangleMesh(Vec<[f32; 3]>, Vec<[u32; 3]>)
}

#[derive(Clone, Copy, Debug)]
pub enum RenderColliderType {
    /// position, rotation, f32 is radius, bool is sensor
    Ball(Option<Vec3>, Option<Vec3>, f32, bool),
    /// position, rotation, f32s are half-x, half-y and half-z size of collider, bool is sensor
    Cuboid(Option<Vec3>, Option<Vec3>, f32, f32, f32, bool),
    /// position, rotation, first is radius, second is height, bool is sensor
    Capsule(Option<Vec3>, Option<Vec3>, f32, f32, bool),
    /// position, rotation, first is radius, second is height, bool is sensor
    Cylinder(Option<Vec3>, Option<Vec3>, f32, f32, bool),
}

impl RenderColliderType {
    pub fn set_transform(&mut self, position: Vec3, rotation: Vec3) {
        match self {
            RenderColliderType::Ball(col_pos, col_rot, _, _) => {
                *col_pos = Some(position);
                *col_rot = Some(rotation);
            },
            RenderColliderType::Cuboid(col_pos, col_rot, _, _, _, _) => {
                *col_pos = Some(position);
                *col_rot = Some(rotation);
            },
            RenderColliderType::Capsule(col_pos, col_rot, _, _, _) => {
                *col_pos = Some(position);
                *col_rot = Some(rotation);
            },
            RenderColliderType::Cylinder(col_pos, col_rot, _, _, _) => {
                *col_pos = Some(position);
                *col_rot = Some(rotation);
            },
        }
    }
}

#[derive(Debug)]
pub struct RenderRay {
    pub origin: Vec3,
    pub direction: Vec3
}

pub fn get_body_transformations(body_parameters: ObjectBodyParameters) -> Option<(Vec3, Vec3)> {
    unsafe {
        if let Some(body) = body_parameters.rigid_body_handle {
            match RIGID_BODY_SET.get(body) {
                Some(body) => {
                    let position = (*body.translation()).into();
                    let rot_quat: Quat = (*body.rotation()).into();
                    let rotation = rad_vec_to_deg(rot_quat.to_euler(glam::EulerRot::XYZ).into());

                    Some((position, rotation))
                },
                None => {
                    debugger::error(&format!("get_body_transformations error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle));
                    return None;
                }
            }
        } else {
            debugger::error("get_body_transformations error\nrigid body is None");
            None
        }
    }
}

pub fn set_body_transformations(body_parameters: ObjectBodyParameters, position: Vec3, rotation: Vec3) {
    unsafe {
        if let Some(body) = body_parameters.rigid_body_handle {
            match RIGID_BODY_SET.get_mut(body) {
                Some(_) => {
                    set_body_position(body_parameters, position);
                    set_body_rotation(body_parameters, position);
                },
                None => debugger::error(&format!("set_body_transformations error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle)),
            }
        } else {
            debugger::error(&format!("set_body_transformations error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle));
        }
    } 
}


pub fn set_body_position(body_parameters: ObjectBodyParameters, position: Vec3) {
    if let Some(body) = body_parameters.rigid_body_handle {
        unsafe {
            match RIGID_BODY_SET.get_mut(body) {
                Some(body) => body.set_translation(position.into(), true),
                None => debugger::error(&format!("set_body_position error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle)),
            }
        }
    } else {
        debugger::error(&format!("set_body_position error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle));
    }
}

pub fn get_body_position(body_parameters: ObjectBodyParameters) -> Option<Vec3> {
    if let Some(body) = body_parameters.rigid_body_handle {
        unsafe {
            match RIGID_BODY_SET.get(body) {
                Some(body) => Some((*body.translation()).into()),
                None => {
                    debugger::error(&format!("get_body_position error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle));
                    None
                }
            }
        }
    } else {
        debugger::error(&format!("get_body_position error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle));
        None
    }
}

pub fn set_body_rotation(body_parameters: ObjectBodyParameters, rotation_deg: Vec3) {
    if let Some(body) = body_parameters.rigid_body_handle {
        unsafe {
            match RIGID_BODY_SET.get_mut(body) {
                Some(body) => {
                    let quat = Quat::from_euler(glam::EulerRot::XYZ,
                        deg_to_rad(rotation_deg.x), deg_to_rad(rotation_deg.y), deg_to_rad(rotation_deg.z));
                    body.set_rotation(quat.into(), true);
                }
                None => {
                    debugger::error(&format!("set_body_rotation error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle));
                }
            }
        }
    } else {
        debugger::error(&format!("set_body_rotation error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle));
    }
}

pub fn get_body_rotation(body_parameters: ObjectBodyParameters) -> Option<Vec3> {
    if let Some(body) = body_parameters.rigid_body_handle {
        unsafe {
            match RIGID_BODY_SET.get(body) {
                Some(body) => {
                    let rot_quat: Quat = (*body.rotation()).into();
                    let rotation = rad_vec_to_deg(rot_quat.to_euler(glam::EulerRot::XYZ).into());

                    return Some(rotation);
                }
                None => {
                    debugger::error(&format!("get_body_rotation error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle));
                    return None;
                },
            }
        }
    } else {
        debugger::error(&format!("get_body_rotation error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle));
        None
    }
}



pub fn is_ray_intersecting(ray: Ray, toi: f32, query_filter: QueryFilter) -> bool {
    unsafe {
        if let Some(_) = QUERY_PIPELINE.cast_ray(&RIGID_BODY_SET, &COLLIDER_SET, &ray, toi, true, query_filter) {
            true
        } else {
            false
        }
    }
}

pub fn get_ray_intersaction_position(ray: Ray, toi: f32, query_filter: QueryFilter) -> Option<Vec3> {
    unsafe {
        if let Some((_, toi)) = QUERY_PIPELINE.cast_ray(&RIGID_BODY_SET, &COLLIDER_SET, &ray, toi, true, query_filter) {
            let hit_point = ray.point_at(toi);
            return Some(Vec3::new(hit_point.x, hit_point.y, hit_point.z));
        } else {
            return None
        }
    }
}

/// To use several CollisionGroups at once, use "|" between them.
/// 
/// Example: `CollisionGroups::Group1 | CollisionGroups::Group3` <- using groups 1 and 3 here
#[bitmask(u32)]
pub enum CollisionGroups {
    Group1, Group2, Group3, Group4, Group5, Group6, Group7, Group8, Group9, Group10,
    Group11, Group12, Group13, Group14, Group15, Group16, Group17, Group18, Group19,
    Group20, Group21, Group22, Group23, Group24, Group25, Group26, Group27, Group28,
    Group29, Group30, Group31, Group32,
}


pub fn collider_type_to_render_collider(collider: &BodyColliderType, is_sensor: bool) -> Option<RenderColliderType> {
    return match collider {
        BodyColliderType::Ball(radius) => Some(RenderColliderType::Ball(None, None, *radius, is_sensor)),
        BodyColliderType::Cuboid(x, y, z) => Some(RenderColliderType::Cuboid(None, None, *x, *y, *z, is_sensor)),
        BodyColliderType::Capsule(radius, height) => Some(RenderColliderType::Capsule(None, None, *radius, *height, is_sensor)),
        BodyColliderType::Cylinder(radius, height) => Some(RenderColliderType::Cylinder(None, None, *radius, *height, is_sensor)),
        BodyColliderType::TriangleMesh(_, _) => None,
    }
}
