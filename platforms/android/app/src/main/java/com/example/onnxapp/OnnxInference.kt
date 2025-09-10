package com.example.onnxapp

import android.content.Context
import android.graphics.Bitmap
import android.util.Log
import java.io.ByteArrayOutputStream

data class ClassificationResult(
    val classId: Int,
    val className: String,
    val confidence: Float
)

data class InferenceResult(
    val data: FloatArray,
    val shape: IntArray,
    val isClassification: Boolean,
    val topPredictions: List<ClassificationResult>,
    val inferenceTimeMs: Float = 0f,
    val preprocessingTimeMs: Float = 0f,
    val postprocessingTimeMs: Float = 0f,
    val totalTimeMs: Float = 0f
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (javaClass != other?.javaClass) return false

        other as InferenceResult

        if (!data.contentEquals(other.data)) return false
        if (!shape.contentEquals(other.shape)) return false
        if (isClassification != other.isClassification) return false
        if (topPredictions != other.topPredictions) return false
        if (inferenceTimeMs != other.inferenceTimeMs) return false
        if (preprocessingTimeMs != other.preprocessingTimeMs) return false
        if (postprocessingTimeMs != other.postprocessingTimeMs) return false
        if (totalTimeMs != other.totalTimeMs) return false

        return true
    }

    override fun hashCode(): Int {
        var result = data.contentHashCode()
        result = 31 * result + shape.contentHashCode()
        result = 31 * result + isClassification.hashCode()
        result = 31 * result + topPredictions.hashCode()
        result = 31 * result + inferenceTimeMs.hashCode()
        result = 31 * result + preprocessingTimeMs.hashCode()
        result = 31 * result + postprocessingTimeMs.hashCode()
        result = 31 * result + totalTimeMs.hashCode()
        return result
    }
    
    override fun toString(): String {
        return "InferenceResult(${data.size} elements, ${String.format("%.2f", totalTimeMs)}ms total, " +
                "prep: ${String.format("%.2f", preprocessingTimeMs)}ms, " +
                "inference: ${String.format("%.2f", inferenceTimeMs)}ms, " +
                "post: ${String.format("%.2f", postprocessingTimeMs)}ms)"
    }
}

class OnnxInference private constructor() {
    
    companion object {
        private const val TAG = "OnnxInference"
        private var isLabelsLoaded = false
        private var isModelLoaded = false
        
        init {
            try {
                System.loadLibrary("onnx_inference")
                Log.d(TAG, "ONNX Inference library loaded successfully")
            } catch (e: UnsatisfiedLinkError) {
                Log.e(TAG, "Failed to load ONNX Inference library", e)
                throw RuntimeException("Failed to load ONNX Inference library", e)
            }
        }
        
        @JvmStatic
        fun create(): OnnxInference {
            return OnnxInference()
        }
        
        @JvmStatic
        fun testJNI(): String {
            return try {
                val instance = OnnxInference()
                instance.testJNINative()
            } catch (e: Exception) {
                "JNI Test Failed: ${e.message}"
            }
        }
        
        @JvmStatic
        fun loadImageNetLabels(context: Context): String {
            if (isLabelsLoaded) return "Labels already loaded"
            
            return try {
                val internalFile = java.io.File(context.filesDir, "imagenet_labels.txt")
                context.assets.open("imagenet_labels.txt").use { input ->
                    internalFile.outputStream().use { output ->
                        input.copyTo(output)
                    }
                }
                
                val result = OnnxInference().loadImageNetLabelsNative(internalFile.absolutePath)
                isLabelsLoaded = result.startsWith("Successfully")
                result
            } catch (e: Exception) {
                "Failed to load ImageNet labels: ${e.message}"
            }
        }
        
        @JvmStatic
        fun loadModel(context: Context, modelFileName: String = "resnet50.onnx"): String {
            return try {
                // Copy model from assets to internal storage if needed
                val modelFile = java.io.File(context.cacheDir, modelFileName)
                if (!modelFile.exists()) {
                    context.assets.open(modelFileName).use { input ->
                        modelFile.outputStream().use { output ->
                            input.copyTo(output)
                        }
                    }
                }
                
                val result = OnnxInference().loadModelNative(modelFile.absolutePath)
                isModelLoaded = result.startsWith("Model loaded successfully")
                result
            } catch (e: Exception) {
                "Failed to load model: ${e.message}"
            }
        }
    }
    
    /**
     * Run inference on an image using the currently loaded ONNX model
     * Note: A model must be loaded first using OnnxInference.loadModel()
     * 
     * @param imageBitmap Bitmap of the input image
     * @return InferenceResult containing the model output and predictions
     */
    fun runInference(imageBitmap: Bitmap): InferenceResult? {
        try {
            // Check if a model is loaded
            if (!isModelLoadedNative()) {
                Log.e(TAG, "No model loaded. Call OnnxInference.loadModel() first.")
                return null
            }
            
            // Convert bitmap to byte array (PNG format)
            val outputStream = ByteArrayOutputStream()
            imageBitmap.compress(Bitmap.CompressFormat.PNG, 100, outputStream)
            val imageBytes = outputStream.toByteArray()
            outputStream.close()
            
            val outputData = runInferenceNative(imageBytes) ?: return null

            val isClassification = isClassificationNative()
            val shape = getOutputShapeNative() ?: intArrayOf()

            val topPredictions = if (isClassification) {
                parseTopPredictions(getTopPredictionsJsonNative())
            } else {
                emptyList()
            }

            // Get timing data from the last inference
            val inferenceTime = getInferenceTimeNative()
            val preprocessingTime = getPreprocessingTimeNative()
            val postprocessingTime = getPostprocessingTimeNative()
            val totalTime = getTotalTimeNative()

            return InferenceResult(
                data = outputData,
                shape = shape,
                isClassification = isClassification,
                topPredictions = topPredictions,
                inferenceTimeMs = inferenceTime,
                preprocessingTimeMs = preprocessingTime,
                postprocessingTimeMs = postprocessingTime,
                totalTimeMs = totalTime
            )
        } catch (e: Exception) {
            Log.e(TAG, "Error running inference", e)
            return null
        }
    }
    
    /**
     * Run inference on an image using ONNX model (legacy method for backward compatibility)
     * 
     * @param modelPath Path to the ONNX model file (ignored - uses cached session)
     * @param imageBitmap Bitmap of the input image
     * @return InferenceResult containing the model output and predictions
     */
    @Deprecated("Use runInference(Bitmap) instead. Model path is ignored.")
    fun runInference(
        modelPath: String,
        imageBitmap: Bitmap,
    ): InferenceResult? {
        Log.w(TAG, "Using deprecated runInference with model path. Path is ignored, using cached session.")
        return runInference(imageBitmap)
    }
    
    private fun parseTopPredictions(json: String?): List<ClassificationResult> {
        if (json.isNullOrEmpty()) return emptyList()

        return try {
            // Simple JSON parsing for the prediction format
            // Format: [{"class_id":123,"class_name":"dog","confidence":0.95}, ...]
            val predictions = mutableListOf<ClassificationResult>()

            // Remove brackets and split by objects
            val cleanJson = json.trim().removePrefix("[").removeSuffix("]")
            if (cleanJson.isEmpty()) return emptyList()

            val objects = cleanJson.split("},{").map {
                it.removePrefix("{").removeSuffix("}")
            }

            for (obj in objects) {
                val parts = obj.split(",")
                var classId = 0
                var className = ""
                var confidence = 0f

                for (part in parts) {
                    val keyValue = part.split(":")
                    if (keyValue.size == 2) {
                        val key = keyValue[0].trim().removeSurrounding("\"")
                        val value = keyValue[1].trim()

                        when (key) {
                            "class_id" -> classId = value.toIntOrNull() ?: 0
                            "class_name" -> className = value.removeSurrounding("\"")
                            "confidence" -> confidence = value.toFloatOrNull() ?: 0f
                        }
                    }
                }

                predictions.add(ClassificationResult(classId, className, confidence))
            }

            predictions
        } catch (e: Exception) {
            Log.e(TAG, "Error parsing predictions JSON", e)
            emptyList()
        }
    }

    // Native method declarations - these must match the actual JNI exports in libonnx_inference.so
    private external fun testJNINative(): String
    private external fun runInferenceNative(
        imageBytes: ByteArray,
    ): FloatArray?
    private external fun isClassificationNative(): Boolean
    private external fun getOutputShapeNative(): IntArray?
    private external fun getTopPredictionsJsonNative(): String?
    private external fun getLastError(): String
    private external fun loadImageNetLabelsNative(labelsPath: String): String
    
    // New session management methods
    private external fun loadModelNative(modelPath: String): String
    private external fun isModelLoadedNative(): Boolean  
    private external fun getLoadedModelPathNative(): String
    
    // Timing methods
    private external fun getInferenceTimeNative(): Float
    private external fun getPreprocessingTimeNative(): Float
    private external fun getPostprocessingTimeNative(): Float
    private external fun getTotalTimeNative(): Float

    /**
     * Get the last error message from Rust
     */
    fun getLastErrorMessage(): String {
        return try {
            getLastError()
        } catch (e: Exception) {
            "Failed to get error: ${e.message}"
        }
    }

    /**
     * Check if any model is currently loaded
     */
    fun isModelLoaded(): Boolean {
        return try {
            isModelLoadedNative()
        } catch (e: Exception) {
            false
        }
    }
    
    /**
     * Get the path of the currently loaded model
     */
    fun getLoadedModelPath(): String? {
        return try {
            val path = getLoadedModelPathNative()
            if (path.isEmpty()) null else path
        } catch (e: Exception) {
            null
        }
    }
}