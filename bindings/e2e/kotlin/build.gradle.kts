plugins {
    kotlin("jvm") version "2.0.21"
    application
}
repositories { mavenLocal(); mavenCentral() }
dependencies {
    val revaultVersion = providers.gradleProperty("revaultVersion").getOrElse("0.2.0")
    implementation("dev.onepub:revault-api:$revaultVersion")
    implementation("dev.onepub:revault-api-kotlin:$revaultVersion")
}
sourceSets {
    main {
        kotlin.setSrcDirs(listOf(projectDir))
        java.setSrcDirs(listOf(projectDir.resolve("../java")))
    }
}
java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(
            providers.gradleProperty("javaToolchain").getOrElse("22").toInt()
        ))
    }
}
tasks.withType<JavaCompile>().configureEach { options.release.set(22) }
application {
    mainClass.set("com.onepub.revault.e2e.kotlin.KotlinConformance")
    applicationDefaultJvmArgs = listOf("--enable-native-access=ALL-UNNAMED")
}
