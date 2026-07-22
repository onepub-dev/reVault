// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "RevaultAPI",
    platforms: [.macOS(.v13)],
    products: [
        .library(name: "RevaultAPI", targets: ["RevaultAPI"]),
        .executable(name: "revault-swift-conformance", targets: ["RevaultConformance"]),
    ],
    dependencies: [
        .package(url: "https://github.com/google/flatbuffers.git", exact: "25.2.10"),
    ],
    targets: [
        .systemLibrary(name: "RevaultC", path: "CModule"),
        .target(
            name: "RevaultAPI",
            dependencies: ["RevaultC", .product(name: "FlatBuffers", package: "flatbuffers")],
            path: "Sources/RevaultAPI"
        ),
        .executableTarget(
            name: "RevaultConformance",
            dependencies: ["RevaultAPI"],
            path: "Sources/RevaultConformance"
        ),
        .testTarget(
            name: "RevaultAPITests",
            dependencies: ["RevaultAPI"],
            path: "Tests/RevaultAPITests"
        ),
    ]
)
