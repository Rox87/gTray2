import unittest
import os
import tempfile
import json
from gtray_data_manager import GTrayDataManager

class TestGTrayDataManager(unittest.TestCase):
    def setUp(self):
        self.temp_file = tempfile.NamedTemporaryFile(delete=False, suffix='.json')
        self.temp_file.close()
        self.manager = GTrayDataManager(filepath=self.temp_file.name, sync_legacy=False)

    def tearDown(self):
        if os.path.exists(self.temp_file.name):
            os.remove(self.temp_file.name)

    def test_provider_crud(self):
        # Create/Update provider
        self.manager.set_provider('openai', 'sk-test-123')
        self.assertEqual(self.manager.get_provider_key('openai'), 'sk-test-123')
        
        providers = self.manager.get_providers()
        self.assertIn('openai', providers)
        self.assertEqual(providers['openai']['api_key'], 'sk-test-123')

        # Remove provider
        self.manager.delete_provider('openai')
        self.assertNotIn('openai', self.manager.get_providers())
        self.assertIsNone(self.manager.get_provider_key('openai'))

    def test_model_crud(self):
        # Setup provider
        self.manager.set_provider('openai', 'sk-test-123')
        
        # Create model linked to provider
        self.manager.set_model('gpt-4o-mini', 'openai')
        self.assertEqual(self.manager.get_model_provider('gpt-4o-mini'), 'openai')
        
        models = self.manager.get_models()
        self.assertIn('gpt-4o-mini', models)
        self.assertEqual(models['gpt-4o-mini']['provider'], 'openai')

        # Remove model
        self.manager.delete_model('gpt-4o-mini')
        self.assertNotIn('gpt-4o-mini', self.manager.get_models())

    def test_shortcut_crud_and_full_chain_lookup(self):
        # 1. Provedor (nome e chave)
        self.manager.set_provider('anthropic', 'sk-ant-test')
        
        # 2. Modelo (nome e provedor)
        self.manager.set_model('claude-3-5-sonnet', 'anthropic')
        
        # 3. Atalho (nome, tecla, modelo, system_prompt, extract_blocks e comment_char)
        self.manager.set_shortcut(
            'upgrade', 'Ctrl+Shift+U', 'claude-3-5-sonnet',
            system_prompt='Instrução de teste',
            extract_blocks=True, comment_char='//'
        )
        
        shortcut = self.manager.get_shortcut('upgrade')
        self.assertIsNotNone(shortcut)
        self.assertEqual(shortcut['key'], 'Ctrl+Shift+U')
        self.assertEqual(shortcut['model'], 'claude-3-5-sonnet')
        self.assertEqual(shortcut['system_prompt'], 'Instrução de teste')
        self.assertTrue(shortcut['extract_blocks'])
        self.assertEqual(shortcut['comment_char'], '//')
        self.assertTrue(shortcut['enabled'])

        # Full chain resolution: Atalho -> Modelo -> Provedor -> Chave -> Base URL -> System Prompt -> Extração -> Ativo
        chain = self.manager.resolve_shortcut('upgrade')
        self.assertEqual(chain['shortcut_name'], 'upgrade')
        self.assertEqual(chain['key'], 'Ctrl+Shift+U')
        self.assertEqual(chain['model_name'], 'claude-3-5-sonnet')
        self.assertEqual(chain['provider_name'], 'anthropic')
        self.assertEqual(chain['api_key'], 'sk-ant-test')
        self.assertEqual(chain['system_prompt'], 'Instrução de teste')
        self.assertTrue(chain['extract_blocks'])
        self.assertEqual(chain['comment_char'], '//')
        self.assertTrue(chain['enabled'])

        # Teste de atalho desativado (enabled=False)
        self.manager.set_shortcut(
            'disabled_sc', 'Ctrl+Shift+D', 'claude-3-5-sonnet',
            enabled=False
        )
        sc_disabled = self.manager.get_shortcut('disabled_sc')
        self.assertFalse(sc_disabled['enabled'])
        chain_disabled = self.manager.resolve_shortcut('disabled_sc')
        self.assertFalse(chain_disabled['enabled'])

    def test_provider_base_url_auto_and_custom(self):
        # Provedor conhecido automático (ex: deepseek)
        self.manager.set_provider('deepseek', 'sk-ds-key')
        self.assertEqual(self.manager.get_provider_base_url('deepseek'), 'https://api.deepseek.com')

        # Provedor com base_url customizada
        self.manager.set_provider('local_vllm', 'token-123', base_url='http://localhost:8000/v1')
        self.assertEqual(self.manager.get_provider_base_url('local_vllm'), 'http://localhost:8000/v1')

    def test_persistence_between_instances(self):
        self.manager.set_provider('groq', 'gsk-123')
        self.manager.set_model('llama-3.3-70b', 'groq')
        self.manager.set_shortcut('fast', 'F8', 'llama-3.3-70b')
        self.manager.save()

        # Load fresh manager on same file
        manager2 = GTrayDataManager(filepath=self.temp_file.name)
        self.assertEqual(manager2.get_provider_key('groq'), 'gsk-123')
        self.assertEqual(manager2.get_model_provider('llama-3.3-70b'), 'groq')
        shortcut = manager2.get_shortcut('fast')
        self.assertEqual(shortcut['key'], 'F8')
        self.assertEqual(shortcut['model'], 'llama-3.3-70b')

if __name__ == '__main__':
    unittest.main()
