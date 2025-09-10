import SwiftUI

struct ContentView: View {
    @State private var name: String = ""
    @State private var inputText: String = ""
    @State private var result: String = "Results will appear here..."
    @State private var statusMessage: String = "Loading..."
    @State private var isProcessing: Bool = false
    
    var body: some View {
        NavigationView {
            ScrollView {
                VStack(spacing: 20) {
                    // Header
                    Text("Rust Shared Core Demo")
                        .font(.title)
                        .fontWeight(.bold)
                    
                    // Status
                    Text(statusMessage)
                        .foregroundColor(statusMessage.contains("✓") ? .green : .red)
                        .padding()
                    
                    // Greeting Section
                    VStack(alignment: .leading, spacing: 10) {
                        Text("Greeting Function")
                            .font(.headline)
                        
                        TextField("Enter your name", text: $name)
                            .textFieldStyle(.roundedBorder)
                        
                        Button("Greet") {
                            greetUser()
                        }
                        .buttonStyle(.borderedProminent)
                        .frame(maxWidth: .infinity)
                    }
                    .padding()
                    .background(Color.gray.opacity(0.1))
                    .cornerRadius(10)
                    
                    // Data Processing Section
                    VStack(alignment: .leading, spacing: 10) {
                        Text("Async Data Processing")
                            .font(.headline)
                        
                        TextField("Enter text to process", text: $inputText)
                            .textFieldStyle(.roundedBorder)
                        
                        Button("Process Data") {
                            Task {
                                await processDataAsync()
                            }
                        }
                        .buttonStyle(.borderedProminent)
                        .frame(maxWidth: .infinity)
                        .disabled(isProcessing)
                    }
                    .padding()
                    .background(Color.gray.opacity(0.1))
                    .cornerRadius(10)
                    
                    // Compute Info Section
                    Button("Get Compute Info") {
                        getComputeInfo()
                    }
                    .buttonStyle(.borderedProminent)
                    .frame(maxWidth: .infinity)
                    
                    // Results Section
                    VStack(alignment: .leading, spacing: 10) {
                        Text("Results")
                            .font(.headline)
                        
                        Text(result)
                            .frame(maxWidth: .infinity, minHeight: 100, alignment: .topLeading)
                            .padding()
                            .background(Color.black)
                            .foregroundColor(.white)
                            .cornerRadius(8)
                            .font(.system(.body, design: .monospaced))
                    }
                    .padding()
                    .background(Color.gray.opacity(0.1))
                    .cornerRadius(10)
                }
                .padding()
            }
            .navigationTitle("SharedCore Demo")
            .navigationBarTitleDisplayMode(.inline)
        }
        .onAppear {
            initializeRustLibrary()
        }
    }
    
    private func initializeRustLibrary() {
        // In a real implementation, this would initialize the Rust library
        // For now, we'll simulate the initialization
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
            statusMessage = "✓ Rust library loaded successfully (Placeholder)"
        }
    }
    
    private func greetUser() {
        let nameToGreet = name.isEmpty ? "Anonymous" : name
        // Placeholder implementation - would call the actual Rust function
        result = SharedCore.greet(name: nameToGreet)
    }
    
    private func processDataAsync() async {
        guard !inputText.isEmpty else {
            result = "Error: Please enter some text to process"
            return
        }
        
        isProcessing = true
        result = "Processing..."
        
        // Placeholder implementation - would call the actual Rust function
        let processResult = await SharedCore.processDataAsync(input: inputText)
        result = "Result: \(processResult.result)\nProcessing time: \(processResult.processingTimeMs)ms"
        
        isProcessing = false
    }
    
    private func getComputeInfo() {
        // Placeholder implementation - would call the actual Rust function
        let info = SharedCore.getComputeInfo()
        result = """
        Backend Types: \(info.backendTypes.joined(separator: ", "))
        GPU Available: \(info.gpuAvailable)
        Features: \(info.availableFeatures.joined(separator: ", "))
        """
    }
}

// Placeholder for the actual UniFFI generated bindings
// In a real implementation, this would be generated by uniffi-bindgen
struct SharedCore {
    static func greet(name: String) -> String {
        return "Hello, \(name)! Welcome to the shared Rust core. (Placeholder - UniFFI bindings not yet generated)"
    }
    
    static func processDataAsync(input: String) async -> ProcessResult {
        // Simulate async processing
        try? await Task.sleep(nanoseconds: 100_000_000) // 100ms
        return ProcessResult(
            result: "Processed: \(input.uppercased()) (Placeholder)",
            processingTimeMs: 100
        )
    }
    
    static func getComputeInfo() -> ComputeInfo {
        return ComputeInfo(
            gpuAvailable: true,
            backendTypes: ["CPU", "CPU_WITH_ACCELERATION", "ARM_NEON", "GPU_METAL"],
            availableFeatures: ["Multi-core CPU", "ARM NEON SIMD", "AArch64 (64-bit ARM)", "Hardware GPU", "Metal GPU API"]
        )
    }
}

struct ProcessResult {
    let result: String
    let processingTimeMs: Int64
}

struct ComputeInfo {
    let gpuAvailable: Bool
    let backendTypes: [String]
    let availableFeatures: [String]
}

#Preview {
    ContentView()
}