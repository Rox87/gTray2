import unittest
import os
import tempfile
from gtray_data_manager import GTrayDataManager
from pessoal.config_manager import ConfigManager

class TestConfigAppAndDataManager(unittest.TestCase):
    def test_app_settings_in_data_manager(self):
        with tempfile.NamedTemporaryFile('w+', delete=False, suffix='.json', encoding='utf-8') as f:
            temp_path = f.name
        try:
            dm = GTrayDataManager(filepath=temp_path)
            self.assertEqual(dm.get_app_setting('title'), 'gTray')
            self.assertEqual(dm.get_app_setting('retry_clip'), 140)
            self.assertEqual(dm.get_app_setting('retry_ia'), 14)
            
            # Atualiza uma configuração
            dm.set_app_setting('title', 'gTray Custom')
            self.assertEqual(dm.get_app_setting('title'), 'gTray Custom')
            
            # Recarrega do arquivo
            dm2 = GTrayDataManager(filepath=temp_path)
            self.assertEqual(dm2.get_app_setting('title'), 'gTray Custom')
        finally:
            if os.path.exists(temp_path):
                os.remove(temp_path)

    def test_config_manager_singleton_delegation(self):
        cm = ConfigManager()
        self.assertEqual(cm.get('GTRAY', 'title'), 'gTray')
        self.assertEqual(cm.get_clip_retry(), 140)
        self.assertEqual(cm.get_retry_attempts(), 14)
        self.assertEqual(cm.get('GTRAY', 'tema'), 'dark')
        self.assertIn('assets', cm.get('GTRAY', 'favicon'))

if __name__ == '__main__':
    unittest.main()
