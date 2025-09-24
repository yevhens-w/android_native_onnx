package com.example.sharedcoreapp

import com.example.onnxapp.OnnxInference

import android.Manifest
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.ImageFormat
import android.hardware.camera2.*
import android.media.ImageReader
import android.os.Bundle
import android.os.Handler
import android.os.HandlerThread
import android.util.Size
import android.widget.Toast
import android.widget.ArrayAdapter
import android.widget.AdapterView
import androidx.activity.result.contract.ActivityResultContracts
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import androidx.lifecycle.lifecycleScope
import com.example.sharedcoreapp.databinding.ActivityMainBinding
import kotlinx.coroutines.launch

class MainActivity : AppCompatActivity() {
    private lateinit var binding: ActivityMainBinding

    // ONNX Inference instance
    private var onnxInference: OnnxInference? = null

    // Available models
    private val availableModels = listOf(
        "resnet50.onnx",
        //"resnet50_fp16.onnx",
        //"resnet50_quantized.onnx"

    )
    private var currentSelectedModel = "resnet50.onnx"

    // Camera2 variables
    private var cameraManager: CameraManager? = null
    private var cameraDevice: CameraDevice? = null
    private var captureSession: CameraCaptureSession? = null
    private var imageReader: ImageReader? = null
    private var backgroundThread: HandlerThread? = null
    private var backgroundHandler: Handler? = null
    private var isCameraStarted = false

    // Current frame bitmap variable
    private var currentFrame: Bitmap? = null

    private val requestPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestPermission()
    ) { isGranted: Boolean ->
        if (isGranted) {
            startCameraPreview()
        } else {
            showError(getString(R.string.error_camera_permission))
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        setupUI()
        initializeRustLibrary()
    }

    private fun setupUI() {
        setupModelSpinner()
        
        binding.buttonComputeInfo.setOnClickListener {
            currentFrame?.let { bitmap ->
                testImageProcessingFirst(bitmap)
            } ?: run {
                showError(getString(R.string.error_no_image))
            }
        }

        binding.buttonCamera.setOnClickListener {
            checkCameraPermissionAndOpen()
        }

        binding.buttonCopyResults.setOnClickListener {
            copyResultsToClipboard()
        }
    }

    private fun setupModelSpinner() {
        // Create adapter for spinner
        val adapter = ArrayAdapter(this, android.R.layout.simple_spinner_item, availableModels)
        adapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item)
        
        // Set adapter to spinner
        binding.spinnerModelSelection.adapter = adapter
        
        // Set default selection
        val defaultIndex = availableModels.indexOf(currentSelectedModel)
        if (defaultIndex >= 0) {
            binding.spinnerModelSelection.setSelection(defaultIndex)
        }
        
        // Set selection listener
        binding.spinnerModelSelection.onItemSelectedListener = object : AdapterView.OnItemSelectedListener {
            override fun onItemSelected(parent: AdapterView<*>?, view: android.view.View?, position: Int, id: Long) {
                val selectedModel = availableModels[position]
                if (selectedModel != currentSelectedModel) {
                    currentSelectedModel = selectedModel
                    switchToModel(selectedModel)
                }
            }
            
            override fun onNothingSelected(parent: AdapterView<*>?) {
                // Do nothing
            }
        }
    }

    private fun initializeRustLibrary() {
        try {
            // Test JNI first
            val jniTestResult = OnnxInference.testJNI()
            android.util.Log.d("MainActivity", "JNI Test: $jniTestResult")

            // Initialize the ONNX Inference library
            onnxInference = OnnxInference.create()

            // Load ImageNet labels
            val labelsResult = OnnxInference.loadImageNetLabels(this)
            android.util.Log.d("MainActivity", "Labels: $labelsResult")

            // Load all available models into session cache
            val modelResults = mutableListOf<String>()
            for (modelName in availableModels) {
                val modelResult = OnnxInference.loadModel(this, modelName)
                modelResults.add("$modelName: $modelResult")
                android.util.Log.d("MainActivity", "Model $modelName: $modelResult")
            }

            // Set the default current model (use absolute path)
            val defaultModelFile = java.io.File(this.cacheDir, currentSelectedModel)
            val setCurrentResult = onnxInference?.setCurrentModel(defaultModelFile.absolutePath)
            android.util.Log.d("MainActivity", "Set current model: $setCurrentResult")

            // Get execution provider info
            val executionProviderInfo = onnxInference?.getExecutionProviderInfo() ?: "Unknown"
            android.util.Log.d("MainActivity", "Execution provider: $executionProviderInfo")

            // Get memory usage info
            val memoryUsageInfo = onnxInference?.getMemoryUsageInfo() ?: "Unknown"
            android.util.Log.d("MainActivity", "Memory usage: $memoryUsageInfo")

            val statusText = buildString {
                append("✓ ONNX Inference library loaded successfully\n")
                append("JNI Test: $jniTestResult\n")
                append("Labels: $labelsResult\n")
                append("Hardware: $executionProviderInfo\n")
                append("Memory: $memoryUsageInfo\n")
                append("Models loaded:\n")
                modelResults.forEach { result ->
                    append("  $result\n")
                }
                append("Current model: $setCurrentResult")
            }

            binding.textViewStatus.text = statusText
            binding.textViewStatus.setTextColor(getColor(android.R.color.holo_green_dark))
        } catch (e: Exception) {
            binding.textViewStatus.text = "✗ Failed to load ONNX library: ${e.message}"
            binding.textViewStatus.setTextColor(getColor(android.R.color.holo_red_dark))
            showError("Failed to initialize ONNX library: ${e.message}")
        }
    }

    private fun switchToModel(modelName: String) {
        android.util.Log.d("MainActivity", "Switching to model: $modelName")
        
        onnxInference?.let { inference ->
            // Use absolute path for the model
            val modelFile = java.io.File(this.cacheDir, modelName)
            val result = inference.setCurrentModel(modelFile.absolutePath)
            android.util.Log.d("MainActivity", "Switch result: $result")
            
            // Get updated execution provider and memory info
            val executionProviderInfo = inference.getExecutionProviderInfo()
            val memoryInfo = inference.getMemoryUsageInfo()
            android.util.Log.d("MainActivity", "Execution provider after switch: $executionProviderInfo")
            android.util.Log.d("MainActivity", "Memory usage after switch: $memoryInfo")
            
            // Update UI to show current model, hardware and memory info
            if (result.startsWith("Current model set to:")) {
                val message = "Switched to $modelName\nHardware: $executionProviderInfo\nMemory: $memoryInfo"
                Toast.makeText(this, message, Toast.LENGTH_LONG).show()
            } else {
                showError("Failed to switch model: $result")
            }
        }
    }

    private fun testImageProcessingFirst(bitmap: Bitmap) {
        lifecycleScope.launch {
            try {
                processImageWithOnnx(bitmap)
            } catch (e: Exception) {
                showError(getString(R.string.error_processing_image, e.message))
            }
        }
    }

    private fun processImageWithOnnx(bitmap: Bitmap) {
        lifecycleScope.launch {
            try {
                // Check if model is loaded
                if (onnxInference?.isModelLoaded() != true) {
                    showError(getString(R.string.error_no_model))
                    return@launch
                }

                // Show which model is loaded
                val loadedModelPath = onnxInference?.getLoadedModelPath()
                android.util.Log.d("ONNX", "Using cached model: $loadedModelPath")

                // Run inference using the simplified API (no model path needed)
                android.util.Log.d("ONNX", "Running inference with cached session")

                val result = onnxInference?.runInference(bitmap)
                val errorMessage = if (result == null) {
                    onnxInference?.getLastErrorMessage() ?: "Unknown error occurred"
                } else null

                // Display results
                if (result != null) {
                    val resultText = formatInferenceResult(result, loadedModelPath)
                    binding.textViewResult.text = resultText
                    binding.textViewResult.setTextColor(getColor(android.R.color.black))
                } else {
                    val errorSummary = errorMessage ?: "Inference failed with no error message"
                    android.util.Log.e("ONNX", errorSummary)
                    showError(getString(R.string.error_inference_failed, errorSummary))
                }

            } catch (e: Exception) {
                showError(getString(R.string.error_processing_image, e.message))
            }
        }
    }


    private fun checkCameraPermissionAndOpen() {
        when {
            ContextCompat.checkSelfPermission(
                this, Manifest.permission.CAMERA
            ) == PackageManager.PERMISSION_GRANTED -> {
                if (isCameraStarted) {
                    stopCameraPreview()
                } else {
                    startCameraPreview()
                }
            }

            else -> {
                requestPermissionLauncher.launch(Manifest.permission.CAMERA)
            }
        }
    }

    private fun startBackgroundThread() {
        backgroundThread = HandlerThread("CameraBackground").also { it.start() }
        backgroundHandler = Handler(backgroundThread!!.looper)
    }

    private fun stopBackgroundThread() {
        backgroundThread?.quitSafely()
        try {
            backgroundThread?.join()
            backgroundThread = null
            backgroundHandler = null
        } catch (e: InterruptedException) {
            showError("Error stopping background thread: ${e.message}")
        }
    }

    private fun startCameraPreview() {
        startBackgroundThread()

        cameraManager = getSystemService(Context.CAMERA_SERVICE) as CameraManager

        try {
            val cameraId = cameraManager!!.cameraIdList[0] // Use back camera
            val characteristics = cameraManager!!.getCameraCharacteristics(cameraId)
            val map = characteristics.get(CameraCharacteristics.SCALER_STREAM_CONFIGURATION_MAP)!!

            val size = Size(640, 480) // Set a reasonable size

            imageReader = ImageReader.newInstance(size.width, size.height, ImageFormat.JPEG, 1)
            imageReader!!.setOnImageAvailableListener(imageAvailableListener, backgroundHandler)

            if (ContextCompat.checkSelfPermission(
                    this, Manifest.permission.CAMERA
                ) == PackageManager.PERMISSION_GRANTED
            ) {
                cameraManager!!.openCamera(cameraId, stateCallback, backgroundHandler)
            } else {
                showError("Camera permission not granted")
            }

        } catch (e: Exception) {
            showError("Failed to start camera: ${e.message}")
        }
    }

    private fun stopCameraPreview() {
        try {
            captureSession?.close()
            captureSession = null

            cameraDevice?.close()
            cameraDevice = null

            imageReader?.close()
            imageReader = null

            stopBackgroundThread()

            isCameraStarted = false
            binding.buttonCamera.text = getString(R.string.button_start_camera)
            binding.textViewResult.text = getString(R.string.status_camera_stopped)

        } catch (e: Exception) {
            showError("Error stopping camera: ${e.message}")
        }
    }

    private val stateCallback = object : CameraDevice.StateCallback() {
        override fun onOpened(camera: CameraDevice) {
            cameraDevice = camera
            createCaptureSession()
        }

        override fun onDisconnected(camera: CameraDevice) {
            camera.close()
            cameraDevice = null
        }

        override fun onError(camera: CameraDevice, error: Int) {
            camera.close()
            cameraDevice = null
            showError("Camera error: $error")
        }
    }

    private fun createCaptureSession() {
        try {
            val surfaces = listOf(imageReader!!.surface)

            cameraDevice!!.createCaptureSession(
                surfaces, object : CameraCaptureSession.StateCallback() {
                    override fun onConfigured(session: CameraCaptureSession) {
                        captureSession = session
                        startRepeatingRequest()
                    }

                    override fun onConfigureFailed(session: CameraCaptureSession) {
                        showError("Failed to configure camera session")
                    }
                }, backgroundHandler
            )

        } catch (e: Exception) {
            showError("Error creating capture session: ${e.message}")
        }
    }

    private fun startRepeatingRequest() {
        try {
            val captureRequestBuilder =
                cameraDevice!!.createCaptureRequest(CameraDevice.TEMPLATE_PREVIEW)
            captureRequestBuilder.addTarget(imageReader!!.surface)

            val captureRequest = captureRequestBuilder.build()
            captureSession!!.setRepeatingRequest(captureRequest, null, backgroundHandler)

            isCameraStarted = true
            binding.buttonCamera.text = getString(R.string.button_stop_camera)
            binding.textViewResult.text = getString(R.string.status_camera_started)

        } catch (e: Exception) {
            showError("Error starting repeating request: ${e.message}")
        }
    }

    private val imageAvailableListener = ImageReader.OnImageAvailableListener { reader ->
        val image = reader.acquireLatestImage()

        if (image != null) {
            // Convert Image to Bitmap and store in currentFrame variable
            val buffer = image.planes[0].buffer
            val bytes = ByteArray(buffer.remaining())
            buffer.get(bytes)

            currentFrame = BitmapFactory.decodeByteArray(bytes, 0, bytes.size)

            // Display the frame on UI thread
            runOnUiThread {
                currentFrame?.let { bitmap ->
                    binding.imageViewCamera.setImageBitmap(bitmap)
                }
            }

            image.close()
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        stopCameraPreview()

        // Clean up ONNX instance
        onnxInference = null
    }

    private fun showError(message: String) {
        binding.textViewResult.text = "Error: $message"
        binding.textViewResult.setTextColor(getColor(android.R.color.holo_red_dark))
        Toast.makeText(this, message, Toast.LENGTH_SHORT).show()
    }

    private fun copyResultsToClipboard() {
        val resultText = binding.textViewResult.text.toString()
        if (resultText.isNotEmpty() && resultText != getString(R.string.results_placeholder)) {
            val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
            val clip = ClipData.newPlainText("Inference Results", resultText)
            clipboard.setPrimaryClip(clip)
            Toast.makeText(this, "Results copied to clipboard", Toast.LENGTH_SHORT).show()
        } else {
            Toast.makeText(this, "No results to copy", Toast.LENGTH_SHORT).show()
        }
    }

    private fun formatInferenceResult(result: com.example.onnxapp.InferenceResult, modelPath: String?): String {
        val modelName = modelPath?.substringAfterLast("/") ?: "Unknown"
        return if (result.isClassification && result.topPredictions.isNotEmpty()) {
            formatClassificationResult(result, modelName)
        } else {
            formatGenericResult(result, modelName)
        }
    }

    private fun formatClassificationResult(result: com.example.onnxapp.InferenceResult, modelName: String): String {
        return buildString {
            append("Model: $modelName\n")
            
            // Add hardware info
            val hardwareInfo = onnxInference?.getExecutionProviderInfo() ?: "Unknown"
            append("Hardware: $hardwareInfo\n")
            
            // Add memory info
            val memoryInfo = onnxInference?.getMemoryUsageInfo() ?: "Unknown"
            append("Memory: $memoryInfo\n")
            
            append(formatTimingInfo(result))
            append("\nTop predictions:\n")
            result.topPredictions.take(3).forEach { pred ->
                append("${pred.className}: ${"%.2f".format(pred.confidence * 100)}%\n")
            }
        }
    }

    private fun formatGenericResult(result: com.example.onnxapp.InferenceResult, modelName: String): String {
        return buildString {
            append("Model: $modelName\n")
            
            // Add hardware info
            val hardwareInfo = onnxInference?.getExecutionProviderInfo() ?: "Unknown"
            append("Hardware: $hardwareInfo\n")
            
            // Add memory info
            val memoryInfo = onnxInference?.getMemoryUsageInfo() ?: "Unknown"
            append("Memory: $memoryInfo\n")
            
            append(formatTimingInfo(result))
            append("Output shape: ${result.shape.contentToString()}\n")
            append("Data size: ${result.data.size}")
        }
    }

    private fun formatTimingInfo(result: com.example.onnxapp.InferenceResult): String {
        return buildString {
            append("Timing: ${"%.2f".format(result.totalTimeMs)}ms total\n")
            append("  • Preprocessing: ${"%.2f".format(result.preprocessingTimeMs)}ms\n")
            append("  • Inference: ${"%.2f".format(result.inferenceTimeMs)}ms\n")
            append("  • Postprocessing: ${"%.2f".format(result.postprocessingTimeMs)}ms\n")
        }
    }
}

// Use the actual Rust bindings generated by our script
// The SharedCore, ProcessResult, and ComputeInfo are now provided by SharedCoreBindings.kt