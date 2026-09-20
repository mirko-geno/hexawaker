use burn::prelude::*;
use burn_vision::{Nms, NmsOptions, VisionBackend};
use image::{
    Rgb,
    RgbImage,
    imageops::FilterType
};
use imageproc::{drawing, rect::Rect};

use crate::sleep_person_yolo26n;


#[derive(Debug)]
pub struct Detection {
    pub confidence: f32,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}


fn adapt_image<B: Backend>(device: &B::Device, image: RgbImage) -> Tensor<B, 4> {
    // Resize to YOLO input resolution
    let image = image::imageops::resize(
        &image, 640, 640, FilterType::Triangle);

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

    let image = Tensor::<B, 1>::from_floats(input.as_slice(), &device)
        .reshape([1, 3, 640, 640]);

    image
}


fn detect<B: Backend + VisionBackend>(device: &B::Device, model_output: Tensor<B, 3>
) -> Vec<Detection> {
    let output_data = model_output.to_data();
    let values = output_data.as_slice::<f32>().unwrap();

    let num_candidates = model_output.dims()[2];

    let mut boxes = Vec::with_capacity(num_candidates * 4);
    let mut scores = Vec::with_capacity(num_candidates);

    for candidate in 0..num_candidates {
        let cx = values[candidate];
        let cy = values[num_candidates + candidate];
        let w = values[2 * num_candidates + candidate];
        let h = values[3 * num_candidates + candidate];

        let x1 = cx - w / 2.0;
        let y1 = cy - h / 2.0;
        let x2 = cx + w / 2.0;
        let y2 = cy + h / 2.0;

        boxes.extend_from_slice(&[
            x1,
            y1,
            x2,
            y2,
        ]);

        scores.push(values[4 * num_candidates + candidate]);
    }

    let boxes = Tensor::<B, 1>::from_floats(
        boxes.as_slice(),
        &device,
    ).reshape([num_candidates, 4]);

    let scores = Tensor::<B, 1>::from_floats(
        scores.as_slice(),
        &device,
    );

    let options = NmsOptions {
        iou_threshold: 0.50,
        score_threshold: 0.25,
        max_output_boxes: 0,
    };

    let keep: Tensor<B, 1, Int> = boxes.clone().nms(scores.clone(), options);

    let kept_boxes: Tensor<B, 2> = boxes.select(0, keep.clone());
    let kept_scores: Tensor<B, 1> = scores.select(0, keep);

    let boxes_data = kept_boxes.into_data();
    let scores_data = kept_scores.into_data();

    let flat_boxes = boxes_data.as_slice::<f32>().unwrap();
    let flat_scores = scores_data.as_slice::<f32>().unwrap();

    let detections = flat_scores.into_iter()
        .enumerate()
        .map(|(i, &confidence)| {
            let offset = i * 4;
            Detection {
                confidence,
                x1: flat_boxes[offset],
                y1: flat_boxes[offset + 1],
                x2: flat_boxes[offset + 2],
                y2: flat_boxes[offset + 3],
            }
        })
        .collect::<Vec<Detection>>();

    detections

}


pub fn analyze_image<B:Backend + VisionBackend>(
    device: &B::Device,
    model: &sleep_person_yolo26n::Model::<B>,
    image: RgbImage,
) -> Vec<Detection> {    
    println!("Original image dims: {}x{}", image.width(), image.height());
    
    let image_tensor = adapt_image(device, image.clone());
    println!("Input shape: {:?}", image_tensor.dims());

    let output = model.forward(image_tensor);
    println!("Output shape: {:?}", output.dims());

    let detections = detect(device, output);
    
    let resized = image::imageops::resize(
        &image,
        640,
        640,
        image::imageops::FilterType::Triangle,
    );
    draw_detections(
        resized,
        &detections,
        "model/debug.jpg",
    );

    detections
}


pub fn draw_detections(mut image: image::RgbImage, detections: &[Detection], path: &str) {
    for detection in detections {
        let x1 = detection.x1.round() as i32;
        let y1 = detection.y1.round() as i32;
        let x2 = detection.x2.round() as i32;
        let y2 = detection.y2.round() as i32;

        let width = (x2 - x1).max(0) as u32;
        let height = (y2 - y1).max(0) as u32;

        println!("x1:{x1}, x2:{x2}, y1:{y1}, y2:{y2}, w:{width}, h:{height}");

        let rect = Rect::at(x1, y1).of_size(width, height);

        drawing::draw_hollow_rect_mut(
            &mut image,
            rect,
            Rgb([255, 0, 0]),
        );
    }

    image.save(path).expect("Failed to save debug image");
}