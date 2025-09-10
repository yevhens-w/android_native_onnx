plugins {
    id("com.android.application") version "8.2.0"
    id("org.jetbrains.kotlin.android") version "1.9.20"
}

android {
    namespace = "com.example.sharedcoreapp"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.example.sharedcoreapp"
        minSdk = 24
        targetSdk = 34
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    kotlinOptions {
        jvmTarget = "1.8"
    }

    buildFeatures {
        viewBinding = true
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.12.0")
    implementation("androidx.appcompat:appcompat:1.6.1")
    implementation("com.google.android.material:material:1.11.0")
    implementation("androidx.constraintlayout:constraintlayout:2.1.4")
    implementation("androidx.lifecycle:lifecycle-viewmodel-ktx:2.7.0")
    implementation("androidx.lifecycle:lifecycle-livedata-ktx:2.7.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.7.3")
    
    // JNA for UniFFI bindings
    implementation("net.java.dev.jna:jna:5.13.0@aar")
    
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.1")
}

// Copy Rust libraries to the appropriate directories
tasks.register("copyRustLibs") {
    doLast {
        copy {
            from("../../target/aarch64-linux-android/release")
            into("src/main/jniLibs/arm64-v8a")
            include("libshared_core.so")
        }
        copy {
            from("../../target/armv7-linux-androideabi/release")
            into("src/main/jniLibs/armeabi-v7a")
            include("libshared_core.so")
        }
        copy {
            from("../../target/x86_64-linux-android/release")
            into("src/main/jniLibs/x86_64")
            include("libshared_core.so")
        }
    }
}

tasks.named("preBuild") {
    dependsOn("copyRustLibs")
}