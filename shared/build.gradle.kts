plugins {
    kotlin("multiplatform")
    id("org.jetbrains.compose") version "1.5.11"
}

// Kept in lockstep with Cargo.toml by scripts/sync_version.sh.
version = "0.3.5"

kotlin {
    jvm("desktop")

    sourceSets {
        val commonMain by getting {
            dependencies {
                implementation(compose.runtime)
                implementation(compose.foundation)
                implementation(compose.material)
            }
        }
        val desktopMain by getting {
            dependencies {
                implementation(compose.desktop.currentOs)
                implementation("net.java.dev.jna:jna:5.14.0")
                // Wire generated UniFFI Kotlin bindings from desktop_bridge
                compileOnly(files("../desktop_bridge/target/generated-sources/uniffi/kotlin"))
                // NOTE: UniFFI Kotlin bindings require JNA to be added at runtime, but for compile time the files are enough.
            }
            // Include generated sources in compilation
            kotlin.srcDir("../desktop_bridge/target/generated-sources/uniffi/kotlin")
        }
    }
}

compose.desktop {
    application {
        mainClass = "com.scmessenger.shared.MainKt"
    }
}
