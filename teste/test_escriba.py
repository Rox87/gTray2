import unittest
from unittest.mock import patch, MagicMock
from pessoal.Escriba import registrador

class TestEscriba(unittest.TestCase):
    def test_log_shortcuts_formats_and_corrects_spelling(self):
        r = registrador()
        logger = r.get_logger()

        test_shortcuts = {
            "upgrade": {"key": "F8", "model": "deepseek-flash"},
            "python": {"key": "F9", "model": "deepseek-flash"},
            "custom": "Ctrl+Shift+K"
        }

        with patch.object(logger, 'info') as mock_info:
            r.log_shortcuts(shortcuts=test_shortcuts, active_shortcut="F8")

            logged_messages = [call.args[0] for call in mock_info.call_args_list]

            # Verifica cabeçalho
            self.assertTrue(any("Atalhos cadastrados" in msg for msg in logged_messages))

            # Verifica presença de todos os atalhos
            self.assertTrue(any("upgrade: F8" in msg for msg in logged_messages))
            self.assertTrue(any("python: F9" in msg for msg in logged_messages))
            self.assertTrue(any("custom: Ctrl+Shift+K" in msg for msg in logged_messages))

            # Garante que as grafias incorretas não foram utilizadas
            for msg in logged_messages:
                self.assertNotIn("precione", msg.lower())
                self.assertNotIn("editavel", msg)

    def test_log_shortcuts_fallback_without_shortcuts_param(self):
        r = registrador()
        logger = r.get_logger()

        with patch.object(logger, 'info') as mock_info:
            r.log_shortcuts()
            logged_messages = [call.args[0] for call in mock_info.call_args_list]

            self.assertTrue(any("Atalhos cadastrados" in msg for msg in logged_messages))
            for msg in logged_messages:
                self.assertNotIn("precione", msg.lower())
                self.assertNotIn("editavel", msg)

if __name__ == '__main__':
    unittest.main()
