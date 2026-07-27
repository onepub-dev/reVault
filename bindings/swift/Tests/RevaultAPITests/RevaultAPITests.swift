import XCTest
@testable import RevaultAPI

final class RevaultAPITests: XCTestCase {
    func testPublicModuleExportsVaultFacade() {
        XCTAssertEqual(String(describing: Vault.self), "Vault")
    }
}
