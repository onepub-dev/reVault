plugins {
    kotlin("jvm") version "2.0.21"
    id("com.vanniktech.maven.publish") version "0.35.0"
}

group = "dev.onepub"
version = "0.2.0"

repositories { mavenLocal(); mavenCentral() }
dependencies { api("dev.onepub:revault-api:0.2.0") }
java { toolchain { languageVersion.set(JavaLanguageVersion.of(22)) } }
tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
    compilerOptions.allWarningsAsErrors.set(true)
}

mavenPublishing {
    publishToMavenCentral(automaticRelease = true, validateDeployment = true)
    if (providers.gradleProperty("signingInMemoryKey").isPresent) {
        signAllPublications()
    }
    coordinates("dev.onepub", "revault-api-kotlin", version.toString())
    pom {
        name.set("reVault Kotlin bindings")
        description.set("Idiomatic Kotlin classes for the complete reVault API")
        url.set("https://github.com/onepub-dev/reVault")
        licenses {
            license {
                name.set("reVault Source Available License 1.0")
                url.set("https://github.com/onepub-dev/reVault/blob/master/rust/revault_lockbox_api/LICENSE")
            }
        }
        scm {
            connection.set("scm:git:https://github.com/onepub-dev/reVault.git")
            developerConnection.set("scm:git:ssh://git@github.com/onepub-dev/reVault.git")
            url.set("https://github.com/onepub-dev/reVault")
        }
        developers {
            developer {
                id.set("onepub")
                name.set("OnePub")
                email.set("bsutton@onepub.dev")
                organization.set("OnePub")
                organizationUrl.set("https://onepub.dev")
            }
        }
    }
}
