from setuptools import Distribution, setup
from setuptools.command.bdist_wheel import bdist_wheel


class BinaryDistribution(Distribution):
    """Tag wheels as platform-specific because they carry a native library."""

    def has_ext_modules(self) -> bool:
        return True


class PlatformWheel(bdist_wheel):
    """Build one Python-3 ABI-independent wheel for the current platform."""

    def finalize_options(self) -> None:
        super().finalize_options()
        self.root_is_pure = False

    def get_tag(self) -> tuple[str, str, str]:
        _python, _abi, platform = super().get_tag()
        return "py3", "none", platform


setup(distclass=BinaryDistribution, cmdclass={"bdist_wheel": PlatformWheel})
