use gdnative::prelude::*;
// Macro that creates the entry-points of the dynamic library.
godot_init!(init);

/// The HelloWorld "class"
#[derive(NativeClass)]
#[inherit(Node)]
pub struct FaceHelper;

#[repr(C)]
#[derive(ToVariant,Copy, Clone)]
pub struct FaceData{
    index: i64,
    position: Vector3,
}

// Function that registers all exposed classes to Godot
fn init(handle: InitHandle) {
    // Register the new `HelloWorld` type we just declared.
    handle.add_class::<FaceHelper>();
}

// You may add any number of ordinary `impl` blocks as you want. However, ...
impl FaceHelper {
    /// The "constructor" of the class.
    fn new(_owner: &Node) -> Self {
        FaceHelper
    }
}

// Only __one__ `impl` block can have the `#[methods]` attribute, which
// will generate code to automatically bind any exported methods to Godot.
#[methods]
impl FaceHelper {

    // To make a method known to Godot, use the #[export] attribute.
    // In Godot, script "classes" do not actually inherit the parent class.
    // Instead, they are "attached" to the parent object, called the "owner".
    //
    // In order to enable access to the owner, it is passed as the second
    // argument to every single exposed method. As a result, all exposed
    // methods MUST have `owner: &BaseClass` as their second arguments,
    // before all other arguments in the signature.
    #[export]
    fn _ready(&self, _owner: &Node) {
        // The `godot_print!` macro works like `println!` but prints to the Godot-editor
        // output tab as well.
        godot_print!("Face helper initialized!");
    }

    #[export]
    fn is_inside_triangle(&self, _owner: &Node, point: Vector3, face_index: i64, mesh_tool_ref: Ref<gdnative::api::mesh_data_tool::MeshDataTool>) -> bool{
        let mesh_tool = unsafe{
            mesh_tool_ref.assume_safe()
        };

        //https://math.stackexchange.com/a/544947
        //barycentric coordinates
        //TODO: make better variable names


        let normal = mesh_tool.get_face_normal(face_index);

        let p1 = mesh_tool.get_vertex(mesh_tool.get_face_vertex(face_index,0));
        let p2 = mesh_tool.get_vertex(mesh_tool.get_face_vertex(face_index,1));
        let p3 = mesh_tool.get_vertex(mesh_tool.get_face_vertex(face_index,2));

        let plane = Plane::from_points(p1,p2,p3).unwrap();
        let plane_projected_point = plane.project(point);
        let u = p2 - p1;
        let v = p3 - p1;
        let n = u.cross(v);
        let w = plane_projected_point - p1;

        let gamma = (u.cross(w).dot(n)) / (n.dot(n));
        let beta = (w.cross(v).dot(n)) / (n.dot(n));
        let alpha = 1.0 - gamma - beta;

        let epsilon = 0.01;
        let below_plane = normal.dot(point) - plane.d < -0.1;
        let output = !below_plane &&
            -epsilon <= alpha && alpha <= 1.0 + epsilon &&
            -epsilon <= beta && beta <= 1.0 + epsilon &&
            -epsilon <= gamma && gamma <= 1.0 + epsilon;
        return output

    }

    #[export]
    fn get_closest_face(&self, _owner: &Node, position: Vector3, mesh_tool_ref: Ref<gdnative::api::mesh_data_tool::MeshDataTool>) -> FaceData{
        let mesh_tool = unsafe{
            mesh_tool_ref.assume_safe()
        };

        let mut closest_position = self.get_face_position(_owner, 0, mesh_tool_ref.clone());
        let mut closest_distance = closest_position.distance_to(position);
        let mut closest_index = 0;
        for face_index in 0..mesh_tool.get_face_count(){
            let face_position = self.get_face_position(_owner, face_index, mesh_tool_ref.clone());
            let face_distance = face_position.distance_to(position);
            if face_distance < closest_distance{
                closest_position = face_position;
                closest_distance = face_distance;
                closest_index = face_index;
            }

        }
        return FaceData{index: closest_index, position: closest_position}
    }


    #[export]
    fn get_standing_face(&self, _owner: &Node, position: Vector3, mesh_tool_ref: Ref<gdnative::api::mesh_data_tool::MeshDataTool>) -> FaceData{
        let mesh_tool = unsafe{
            mesh_tool_ref.assume_safe()
        };

        let mut valid_faces = vec![];
        for face_index in 0..mesh_tool.get_face_count() {
            let face_position = self.get_face_position(_owner,face_index, mesh_tool_ref.clone());
            if self.is_inside_triangle(_owner,position, face_index, mesh_tool_ref.clone()) {
                valid_faces.push(FaceData{index:face_index, position: face_position});
            }
        }
        if valid_faces.len() > 1{
            let mut closest_face =  valid_faces[0];
            let mut closest_face_distance = valid_faces[0].position.distance_squared_to(position);
            for i in 1..valid_faces.len(){
                if valid_faces[i].position.distance_squared_to(position) < closest_face_distance{
                    closest_face = valid_faces[i];
                    closest_face_distance = valid_faces[i].position.distance_squared_to(position);
                }
            }
            return closest_face
        }

        if valid_faces.len() > 0{
            return valid_faces[0]
        }
        godot_print!("Failed to find triangle");
        return FaceData{index: -1, position: Vector3::zero()} //shouldn't reach
    }

    #[export]
    fn get_face_position(&self, _owner: &Node, face_index: i64, mesh_tool_ref: Ref<gdnative::api::mesh_data_tool::MeshDataTool>) -> Vector3{
        let mesh_tool = unsafe{
            mesh_tool_ref.assume_safe()
        };

        let mut sum = Vector3::zero();

        for i in 0..3{
            sum += mesh_tool.get_vertex(mesh_tool.get_face_vertex(face_index, i));
        }

        return sum / 3.0;

    }

}