vcpkg_download_distfile(ARCHIVE
  URLS "https://github.com/onepub-dev/reVault/releases/download/revault-api-v${VERSION}/revault-api-native-${VCPKG_TARGET_TRIPLET}-${VERSION}.tar.gz"
  FILENAME "revault-api-${VERSION}-${VCPKG_TARGET_TRIPLET}.tar.gz"
  SHA512 "${REVAULT_API_ARCHIVE_SHA512}")
vcpkg_extract_source_archive(SOURCE_PATH ARCHIVE "${ARCHIVE}")
file(INSTALL "${SOURCE_PATH}/include/revault_api.h" DESTINATION "${CURRENT_PACKAGES_DIR}/include")
file(INSTALL "${SOURCE_PATH}/lib/" DESTINATION "${CURRENT_PACKAGES_DIR}/lib")
file(INSTALL "${SOURCE_PATH}/LICENSE" DESTINATION "${CURRENT_PACKAGES_DIR}/share/${PORT}" RENAME copyright)
