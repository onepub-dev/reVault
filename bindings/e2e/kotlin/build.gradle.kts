plugins {
    kotlin("jvm") version "2.0.21"
    application
}
repositories { mavenLocal(); mavenCentral() }
dependencies {
    implementation("dev.onepub:revault-api:0.2.0")
    implementation("dev.onepub:revault-api-kotlin:0.2.0")
}
sourceSets {
    main {
        kotlin.setSrcDirs(listOf(projectDir))
        java.setSrcDirs(listOf(projectDir.resolve("../java")))
    }
}
java { toolchain { languageVersion.set(JavaLanguageVersion.of(22)) } }
tasks.withType<JavaCompile>().configureEach { options.release.set(22) }
application {
    mainClass.set("com.onepub.revault.e2e.kotlin.KotlinConformance")
    applicationDefaultJvmArgs = listOf("--enable-native-access=ALL-UNNAMED")
}
