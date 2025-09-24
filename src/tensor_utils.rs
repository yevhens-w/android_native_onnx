use crate::constants::{IMAGE_HEIGHT, IMAGE_WIDTH};
use crate::errors::{InferenceError, InferenceResult};
use ort::{value::{Value, ValueRef}, tensor::TensorElementType};

pub struct TensorUtils;

impl TensorUtils {
    pub fn create_input_tensor(
        input_data: Vec<f32>,
        tensor_type: TensorElementType,
    ) -> InferenceResult<Value> {
        match tensor_type {
            TensorElementType::Float16 => {
                let f16_data: Vec<half::f16> = input_data
                    .iter()
                    .map(|&x| half::f16::from_f32(x))
                    .collect();

                Value::from_array(([1, 3, IMAGE_HEIGHT as i64, IMAGE_WIDTH as i64], f16_data))
                    .map_err(|e| InferenceError::inference_failed(format!("Failed to create f16 input tensor: {:?}", e)))
                    .map(|v| v.into())
            }
            TensorElementType::Float32 => {
                Value::from_array(([1, 3, IMAGE_HEIGHT as i64, IMAGE_WIDTH as i64], input_data))
                    .map_err(|e| InferenceError::inference_failed(format!("Failed to create f32 input tensor: {:?}", e)))
                    .map(|v| v.into())
            }
            _ => Err(InferenceError::inference_failed(format!(
                "Unsupported input tensor type: {:?}",
                tensor_type
            ))),
        }
    }

    pub fn extract_output_data(output: &ValueRef) -> InferenceResult<Vec<f32>> {
        if let Ok((_output_shape, data_slice)) = output.try_extract_tensor::<f32>() {
            Ok(data_slice.to_vec())
        } else if let Ok((_output_shape, data_slice)) = output.try_extract_tensor::<half::f16>() {
            Ok(data_slice.iter().map(|&x| x.to_f32()).collect())
        } else {
            Err(InferenceError::output_processing_failed(
                "Failed to extract tensor data: unsupported output type",
            ))
        }
    }

    pub fn get_tensor_shape(output: &ValueRef) -> Vec<usize> {
        output.shape().iter().map(|&x| x as usize).collect()
    }
}