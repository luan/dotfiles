import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class NormalizeLinksTest(unittest.TestCase):
    def test_owned_alias_links_are_normalized_and_other_files_are_preserved(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            repo = root / "repo"
            repo.mkdir()
            subprocess.run(["git", "init", "-q", str(repo)], check=True)
            for name in ("fish/config.fish", "mise/config.toml", "zed/settings.json"):
                source = repo / "xdg-configs" / name
                source.parent.mkdir(parents=True, exist_ok=True)
                source.write_text("managed\n")
            subprocess.run(["git", "-C", str(repo), "add", "."], check=True)
            external = root / "vendor.fish"
            external.write_text("vendor completion\n")
            vendor_link = repo / "xdg-configs/fish/vendor.fish"
            vendor_link.symlink_to(external)
            script = repo / "normalize-links.py"
            script.write_bytes(Path(__file__).with_name("normalize-links.py").read_bytes())
            alias = root / "old-checkout"
            alias.symlink_to(repo, target_is_directory=True)
            config = root / "home/.config"
            config.mkdir(parents=True)
            (config / "fish").symlink_to(alias / "xdg-configs/fish")
            unrelated = root / "personal"
            unrelated.mkdir()
            (config / "mise").symlink_to(unrelated)
            (config / "zed").mkdir()
            (config / "zed/settings.json").write_text("personal\n")
            subprocess.run([sys.executable, str(script), str(root / "home")], check=True)
            self.assertEqual(os.readlink(config / "fish"), "../../repo/xdg-configs/fish")
            self.assertEqual((config / "fish/config.fish").read_text(), "managed\n")
            self.assertEqual(os.readlink(vendor_link), str(external))
            self.assertEqual((config / "mise").resolve(), unrelated)
            self.assertEqual((config / "zed/settings.json").read_text(), "personal\n")
