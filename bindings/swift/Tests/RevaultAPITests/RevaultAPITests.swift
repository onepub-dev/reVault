import XCTest
@testable import RevaultAPI

final class RevaultAPITests: XCTestCase {
    func testPublicModuleExportsVaultFacade() {
        XCTAssertEqual(String(describing: Vault.self), "Vault")
        XCTAssertEqual(String(describing: Revault.self), "Revault")
        XCTAssertEqual(String(describing: AgentSession.self), "AgentSession")
    }
}
