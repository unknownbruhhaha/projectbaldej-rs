use glium::Display;

use super::{Object, Transform, ObjectId, generate_object_id};

#[derive(Debug)]
pub struct EmptyObject {
    pub name: String,
    pub transform: Transform,
    pub parent_transform: Option<Transform>,
    pub children: Vec<Box<dyn Object>>,
    id: ObjectId
}

impl EmptyObject {
    pub fn new(name: &str) -> Self {
        EmptyObject { transform: Transform::default(), children: vec![], name: name.to_string(), parent_transform: None, id: generate_object_id() }
    }
}


impl Object for EmptyObject {
    fn start(&mut self) { }

    fn update(&mut self) { }

    fn render(&mut self, _display: &mut Display, _target: &mut glium::Frame) { }

    fn get_children_list(&self) -> &Vec<Box<dyn Object>> {
        &self.children
    }

    fn get_children_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.children
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_object_type(&self) -> &str {
        "EmptyObject"
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn get_local_transform(&self) -> Transform {
        self.transform
    }



    fn set_local_transform(&mut self, transform: Transform) {
        self.transform = transform
    }

    fn get_parent_transform(&self) -> Option<Transform> {
        self.parent_transform
    }

    fn set_parent_transform(&mut self, transform: Transform) {
        self.parent_transform = Some(transform);
    }

    fn set_id(&mut self, object_id: ObjectId) {
        self.id = object_id;
    }


    fn get_id(&self) -> &ObjectId {
        &self.id
    }

    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<&str> {
        if name == "test" {
            println!("test message {}", args[0])
        }
        None
    }
}
