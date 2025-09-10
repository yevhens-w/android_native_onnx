FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

# Install system dependencies
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
        build-essential \
        cmake \
        pkg-config \
        libssl-dev \
        git \
        curl \
        wget \
        unzip \
        openjdk-17-jdk \
        python3 \
        python3-pip \
    && apt-get autoremove -y && apt-get clean -y

# Set JAVA_HOME
ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64

# Install Android SDK
ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_SDK_ROOT=$ANDROID_HOME
ENV PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools

RUN mkdir -p $ANDROID_HOME/cmdline-tools && \
    wget -q https://dl.google.com/android/repository/commandlinetools-linux-9477386_latest.zip -O /tmp/cmdline-tools.zip && \
    unzip -q /tmp/cmdline-tools.zip -d $ANDROID_HOME/cmdline-tools && \
    mv $ANDROID_HOME/cmdline-tools/cmdline-tools $ANDROID_HOME/cmdline-tools/latest && \
    rm /tmp/cmdline-tools.zip

# Accept Android SDK licenses and install required packages
RUN yes | $ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager --licenses && \
    $ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager \
        "platform-tools" \
        "platforms;android-34" \
        "build-tools;34.0.0" \
        "ndk;25.2.9519653"

# Set NDK path
ENV ANDROID_NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653
ENV PATH=$PATH:$ANDROID_NDK_HOME

# Install Rust targets for mobile development
RUN rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    i686-linux-android \
    aarch64-apple-ios \
    x86_64-apple-ios \
    aarch64-apple-ios-sim

# Install cargo tools
RUN cargo install \
    cargo-ndk \
    uniffi_bindgen \
    cbindgen

# Set user permissions
RUN chown -R vscode:vscode $ANDROID_HOME

# Switch to vscode user
USER vscode

# Install additional Rust components
RUN rustup component add rustfmt clippy