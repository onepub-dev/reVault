plugins {
    kotlin("jvm") version "2.0.21"
    `maven-publish`
}

group = "dev.onepub"
version = "0.1.0"

repositories { mavenLocal(); mavenCentral() }
dependencies { api("dev.onepub:revault-api:0.1.0") }
sourceSets.main { kotlin.setSrcDirs(listOf(projectDir)) }
java { toolchain { languageVersion.set(JavaLanguageVersion.of(22)) } }

publishing {
    publications {
        create<MavenPublication>("mavenKotlin") {
            from(components["java"])
            artifactId = "revault-api-kotlin"
            pom {
                name.set("reVault Kotlin bindings")
                description.set("Idiomatic Kotlin classes for the complete reVault API")
                url.set("https://github.com/onepub-dev/reVault")
            }
        }
    }
}
