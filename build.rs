use burn_onnx::ModelGen;
// For saving the generated files to the project
/*
use std::env;

fn main(){
    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let model_dir = format!("{project_dir}/model/");
    let onnx_path = format!("{model_dir}/sleep_person_yolo26n.onnx");
        

    ModelGen::new()
        .input(&onnx_path)
        .out_dir(&model_dir)
        .run_from_script();
}
*/
fn main() {
    ModelGen::new()
        .input("model/sleep_person_yolo26n-v2.onnx")
        .out_dir("model/")
        .run_from_script();
}

