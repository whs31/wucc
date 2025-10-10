from conan import ConanFile
from conan.errors import ConanException
import subprocess
import shutil
import os


class WuccConan(ConanFile):
    name = "wucc"
    version = "0.2.0"
    package_type = "application"
    settings = "os", "arch"
    tool_requires = ()

    user = "whs31"
    channel = "dev"

    def _run_command(self, command):
        try:
            result = subprocess.run(
                command,
                check=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
            return result.stdout.strip()
        except subprocess.CalledProcessError as e:
            self.output.error(f"Command failed: {' '.join(command)}\n{e.stderr}")
            return None

    def system_requirements(self):
        if self.is_installed():
            return
        self._install()

    def is_installed(self) -> bool:
        if shutil.which("wucc") is None:
            self.output.info("wucc not found in PATH")
            return False

        version_output = self._run_command(["wucc", "--version"])
        if not version_output:
            self.output.warning("Failed to get wucc version")
            return False

        try:
            installed_version = version_output.split()[1]
            self.output.info(f"Found wucc version: {installed_version}")

            from conan.tools.scm import Version

            if Version(installed_version) < Version(self.version):
                self.output.warning(
                    f"Installed wucc version {installed_version} is older than required {self.version}"
                )
                return False

            self.output.info(
                f"wucc version {installed_version} meets requirement {self.version}"
            )
            return True

        except (IndexError, Exception) as e:
            self.output.warning(f"Failed to parse wucc version: {e}")
            return False

    def _install(self):
        self.output.info("Installing wucc via cargo...")
        if shutil.which("cargo") is None:
            raise ConanException(
                "cargo is required to install wucc but was not found in PATH"
            )
        result = self._run_command(
            ["cargo", "install", "wucc", "--version", self.version]
        )
        if result is None:
            raise ConanException(f"Failed to install wucc {self.version} via cargo")
        self.output.info(f"Successfully installed wucc {self.version}")

    def package_info(self):
        self.output.info("wucc provided via system PATH")
