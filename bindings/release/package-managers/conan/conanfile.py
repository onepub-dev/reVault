from conan import ConanFile
from conan.tools.files import copy, get
import os


class RevaultApiConan(ConanFile):
    name = "revault-api"
    version = "0.2.0"
    package_type = "shared-library"
    settings = "os", "arch", "compiler", "build_type"
    license = "reVault Source Available License 1.0"
    homepage = "https://github.com/onepub-dev/reVault"

    def source(self):
        get(self, **self.conan_data["sources"][str(self.version)][str(self.settings.os)][str(self.settings.arch)])

    def package(self):
        copy(self, "revault_api.h", self.source_folder, os.path.join(self.package_folder, "include"))
        copy(self, "*revault_api*", self.source_folder, os.path.join(self.package_folder, "lib"))
        copy(self, "LICENSE", self.source_folder, os.path.join(self.package_folder, "licenses"))

    def package_info(self):
        self.cpp_info.libs = ["revault_api"]
