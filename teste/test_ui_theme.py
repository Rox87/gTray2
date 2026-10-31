import unittest
from ui_theme import get_theme_stylesheet, get_status_badge_style

class TestUITheme(unittest.TestCase):
    def test_dark_stylesheet_contains_palette(self):
        qss = get_theme_stylesheet('dark')
        self.assertIn('#0F172A', qss.upper())
        self.assertIn('QPushButton', qss)
        self.assertIn('QLineEdit', qss)

    def test_only_dark_palette_present(self):
        from ui_theme import PALETTES
        self.assertIn('dark', PALETTES)
        self.assertNotIn('light', PALETTES)

    def test_default_stylesheet_is_dark(self):
        qss_default = get_theme_stylesheet()
        qss_dark = get_theme_stylesheet('dark')
        self.assertEqual(qss_default, qss_dark)
        self.assertIn('#0F172A', qss_default.upper())

    def test_status_badge_styles(self):
        active_style = get_status_badge_style(True, 'dark')
        self.assertIn('#22C55E', active_style.upper())
        inactive_style = get_status_badge_style(False, 'dark')
        self.assertTrue('#94A3B8' in inactive_style.upper() or '#EF4444' in inactive_style.upper())

if __name__ == '__main__':
    unittest.main()
