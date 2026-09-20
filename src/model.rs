use burn::prelude::*;
use burn_ndarray::NdArray;
use image::imageops::FilterType;

mod sleep_person_yolo26n {
    include!(concat!(env!("OUT_DIR"), "/model/sleep_person_yolo26n.rs"));
}


pub fn image_test() {
    type Backend = NdArray<f32>;
    let device = Default::default();
    
    let model = sleep_person_yolo26n::Model::<Backend>::default();
    println!("Modelo cargado correctamente.");

    // Load image
    let image = image::open("model/test.jpg")
        .expect("No se pudo abrir model/test.jpg")
        .to_rgb8();

    println!("Imagen original: {}x{}", image.width(), image.height());

    // Resize to YOLO input resolution
    let image = image::imageops::resize(&image, 640, 640, FilterType::Triangle);

    // Convert HWC [640, 640, 3] -> CHW [3, 640, 640]
    let mut input = Vec::with_capacity(3 * 640 * 640);

    for channel in 0..3 {
        for y in 0..640 {
            for x in 0..640 {
                let pixel = image.get_pixel(x, y);
                input.push(pixel[channel] as f32 / 255.0);
            }
        }
    }

    let images = Tensor::<Backend, 1>::from_floats(input.as_slice(), &device)
        .reshape([1, 3, 640, 640]);

    println!("Input shape: {:?}", images.dims());

    let output = model.forward(images);

    println!("Output shape: {:?}", output.dims());

    let output_data = output.to_data();
    let values = output_data.as_slice::<f32>().unwrap();

    let num_candidates = 8400;

    for candidate in 0..num_candidates {
        let x = values[candidate];
        let y = values[num_candidates + candidate];
        let w = values[2 * num_candidates + candidate];
        let h = values[3 * num_candidates + candidate];
        let score = values[4 * num_candidates + candidate];

        if score > 0.01 {
            println!(
                "candidate {candidate}: x={x:.2}, y={y:.2}, w={w:.2}, h={h:.2}, score={score:.4}"
            );
        }
    }
}
