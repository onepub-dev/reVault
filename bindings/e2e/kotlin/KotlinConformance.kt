package com.onepub.revault.e2e.kotlin

/**
 * Kotlin entry point for the exhaustive JVM conformance scenario. The Kotlin
 * API is a zero-cost typed facade over the same owned JVM classes, so the
 * shared scenario exercises every Kotlin route without a second ABI layer.
 */
object KotlinConformance {
    @JvmStatic
    fun main(arguments: Array<String>) {
        val runner = Class.forName("com.onepub.revault.e2e.Conformance")
        runner.getMethod("main", Array<String>::class.java).invoke(null, arguments)
    }
}
