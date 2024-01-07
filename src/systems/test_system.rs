use glam::Vec3;
use crate::{managers::{systems::CallList, networking::{Message, self, MessageReliability, SyncObjectMessage, MessageContents}, physics::{BodyType, BodyColliderType, RenderColliderType, CollisionGroups}, input::{self, InputEventType}}, objects::{Object, model_object::ModelObject, empty_object::EmptyObject, ray::Ray}, assets::{model_asset::ModelAsset, shader_asset::ShaderAsset}};
use super::System;

pub struct TestSystem {
    pub is_destroyed: bool,
    pub objects: Vec<Box<dyn Object>>
}

impl TestSystem {
    pub fn new() -> TestSystem {
        TestSystem {
            is_destroyed: false,
            objects: vec![]
        }
    }
}

impl System for TestSystem {
    fn call(&self, _call_id: &str) { }
    fn call_mut(&mut self, _call_id: &str) { }


    fn start(&mut self) {
        //let asset = ModelAsset::from_file("models/test_model.gltf");
        let asset = ModelAsset::from_file("models/knife_test.gltf");
        let mut knife_model = 
            Box::new(ModelObject::new("knife_model", asset.unwrap(), None, ShaderAsset::load_default_shader().unwrap()));
        //println!("start");
        knife_model.set_position(Vec3::new(0.0, 10.0, 6.0), true);

        let mut ground_collider = Box::new(EmptyObject::new("ground_collider"));
        ground_collider.set_position(Vec3::new(0.0, -2.0, 0.0), true);

        let mut ray = Box::new(Ray::new("ray", Vec3::new(0.0, 0.0, 40.0), Some(CollisionGroups::Group3)));
        ray.set_position(Vec3::new(-2.0, -1.5, 0.0), false);
        ray.set_rotation(Vec3::new(0.0, 0.0, 0.0), false);

        if networking::is_server() {
            knife_model.build_object_rigid_body(Some(BodyType::Dynamic(Some(BodyColliderType::Cuboid(0.2, 2.0, 0.2)))), None, 1.0, Some(CollisionGroups::Group3), None);
            ground_collider.build_object_rigid_body(Some(BodyType::Fixed(Some(BodyColliderType::Cuboid(10.0, 0.5, 10.0)))),
                None, 1.0, None, Some(CollisionGroups::Group2 | CollisionGroups::Group1 | CollisionGroups::Group3));
        } else {
            knife_model.build_object_rigid_body(None, Some(RenderColliderType::Cuboid(None, None, 0.2, 2.0, 0.2)), 1.0, None, None);
            ground_collider.build_object_rigid_body(None, Some(RenderColliderType::Cuboid(None, None, 10.0, 0.5, 10.0)), 1.0, None, None);
        }

        self.add_object(knife_model);
        self.add_object(ground_collider);
        self.add_object(ray);

        input::new_bind("forward", vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::W)]);
        input::new_bind("left", vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::A)]);
        input::new_bind("backwards", vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::S)]);
        input::new_bind("right", vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::D)]);
    }

    fn update(&mut self) {
        {
            let obj = self.find_object_mut("knife_model").unwrap();
            let obj_position = obj.get_local_transform();

            if networking::is_server() {
                let _ = self.send_message(MessageReliability::Reliable, Message {
                    receiver: networking::MessageReceiver::Everybody,
                    system_id: self.system_id().into(),
                    message_id: "sync_knife".into(),
                    message: MessageContents::SyncObject(SyncObjectMessage {
                        object_name: "knife_model".into(),
                        transform: obj_position,
                    }),
                });

            }
        }

        //let ray = self.find_object_mut("ray").unwrap();
        //dbg!(ray.call("is_intersecting", vec![]));
    }

    fn render(&mut self) { }



    fn system_id(&self) -> &str {
        "TestSystem"
    }

    fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    fn set_destroyed(&mut self, is_destroyed: bool) {
        self.is_destroyed = is_destroyed;
    }

    fn get_call_list(&self) -> CallList {
        CallList {
            immut_call: vec![],
            mut_call: vec![]
        }
    }

    fn get_objects_list(&self) -> &Vec<Box<dyn Object>> {
        &self.objects
    }
    fn get_objects_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.objects
    }

    fn reg_message(&mut self, message: Message) {
        match message.message {
            networking::MessageContents::SyncObject(sync_msg) => {
                if &sync_msg.object_name == "knife_model" {
                    let object = self.find_object_mut("knife_model");
                    object.unwrap().set_local_transform(sync_msg.transform);
                }
            },
            networking::MessageContents::Custom(_) => (),
        }
    }
}
