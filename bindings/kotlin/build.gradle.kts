plugins {
    kotlin("jvm") version "2.0.21"
    id("com.vanniktech.maven.publish") version "0.35.0"
}

group = "dev.onepub"
version = "0.1.0"

repositories { mavenLocal(); mavenCentral() }
dependencies { api("dev.onepub:revault-api:0.1.0") }
java { toolchain { languageVersion.set(JavaLanguageVersion.of(22)) } }

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
        scm { url.set("https://github.com/onepub-dev/reVault") }
        developers { developer { id.set("onepub"); name.set("OnePub") } }
    }
}
