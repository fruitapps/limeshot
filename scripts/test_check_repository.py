"""Regression coverage for local Markdown and HTML link checks."""
import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import check_repository


class LinkTests(unittest.TestCase):
    def test_destinations_and_titles(self):
        text = r'''
        [plain](docs/file.md)
        [double](docs/file.md "A title (with parentheses)")
        [single](docs/file.md 'A title')
        [parenthesized](docs/file.md (A title))
        [nested](docs/a(b(c)).md "Title")
        [angle](<docs/a (b).md> "Title")
        [escaped](docs/a\(b\).md)
        ![image](images/a(b).png)
        <a href="docs/html.html">HTML</a><img src="images/html.png">
        '''
        self.assertEqual(list(check_repository.link_targets(text)), [
            "docs/file.md", "docs/file.md", "docs/file.md", "docs/file.md",
            "docs/a(b(c)).md", "docs/a (b).md", "docs/a(b).md", "images/a(b).png",
            "docs/html.html", "images/html.png",
        ])

    def test_unclosed_links_are_not_destinations(self):
        for text in ["[bad](docs/a(b.md)", '[bad](<docs/a.md)', '[bad](docs/a.md "title)']:
            with self.subTest(text=text):
                self.assertEqual(list(check_repository.link_targets(text)), [])

    def test_indented_paragraph_continuation_is_checked(self):
        text = 'Paragraph\n    [link](missing.md)\n\t<img src="missing.png">\n'
        self.assertEqual(list(check_repository.link_targets(
            check_repository.without_code_blocks(text)
        )), ["missing.md", "missing.png"])

    def test_checks_paths_without_titles_and_preserves_html_checks(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs/roadmap").mkdir(parents=True)
            (root / "docs/roadmap/features.json").write_text(json.dumps({"features": []}))
            (root / "a(b(c)).md").touch()
            (root / "a b.md").touch()
            (root / "image.png").touch()
            readme = root / "README.md"
            readme.write_text('''[nested](a(b(c)).md "Title")
            [angle](<a b.md> 'Title')
            <img src="image.png"><a href="a(b(c)).md">HTML</a>
            ''')
            output = io.StringIO()
            with patch.object(check_repository, "ROOT", root), patch.object(
                check_repository.subprocess, "check_output", return_value=b"README.md\0"
            ), contextlib.redirect_stdout(output):
                self.assertFalse(check_repository.check())
                readme.write_text(readme.read_text() + '\n[missing](missing(a).md "Title")')
                self.assertTrue(check_repository.check())
            self.assertIn("missing link target missing(a).md", output.getvalue())
            self.assertNotIn('missing link target a(b(c)).md', output.getvalue())

    def test_checks_ignore_code_blocks(self):
        examples = '[example](missing-code.md)\n<img src="missing-code.png">\n'
        blocks = {
            "backticks": "```markdown\n" + examples + "```\n",
            "tildes": "~~~markdown\n" + examples + "~~~\n",
            "spaces": "".join("    " + line for line in examples.splitlines(keepends=True)),
            "tabs": "".join("\t" + line for line in examples.splitlines(keepends=True)),
            "heading before indented code": '## Example\n    [example](missing-code.md)\n',
            "indented heading before tabbed code": '   ###### Example\n\t[example](missing-code.md)\n',
            "indented blank line": '    [example](missing-code.md)\n\n    <img src="missing-code.png">\n',
            "indented fence": "   ~~~markdown\n" + examples + "   ~~~\n",
            "long fence": "````markdown\n```\n" + examples + "`````\n",
            "mismatched fence": "```markdown\n~~~\n" + examples + "```\n",
            "closing fence with text": "```markdown\n```not a closer\n" + examples + "```\n",
            "fence inside indented code": "    ```\n    [example](missing-code.md)\n",
        }
        unclosed = {
            "unclosed backticks": "```markdown\n" + examples,
            "unclosed tildes": "~~~markdown\n" + examples,
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "docs/roadmap").mkdir(parents=True)
            (root / "docs/roadmap/features.json").write_text(json.dumps({"features": []}))
            (root / "existing.md").touch()
            for name, block in (blocks | unclosed).items():
                with self.subTest(block=name):
                    tail = "\n[after](existing.md)\n" if name in blocks else ""
                    readme = root / "README.md"
                    readme.write_text("[before](existing.md)\n\n" + block + tail)
                    output = io.StringIO()
                    with patch.object(check_repository, "ROOT", root), patch.object(
                        check_repository.subprocess, "check_output", return_value=b"README.md\0"
                    ), contextlib.redirect_stdout(output):
                        self.assertFalse(check_repository.check())
                        readme.write_text(readme.read_text().replace("existing.md", "missing-prose.md"))
                        self.assertTrue(check_repository.check())
                    self.assertNotIn("missing link target missing-code", output.getvalue())
                    self.assertEqual(output.getvalue().count("missing link target missing-prose.md"),
                                     2 if name in blocks else 1)


if __name__ == "__main__":
    unittest.main()
