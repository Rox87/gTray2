import unittest
from unittest.mock import patch
import sys
import os
from PyQt5.QtWidgets import QApplication, QLineEdit

class TestGTrayUI(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Cria a aplicação Qt em modo headless/offscreen para testes
        os.environ["QT_QPA_PLATFORM"] = "offscreen"
        if not QApplication.instance():
            cls.app = QApplication(sys.argv)
        else:
            cls.app = QApplication.instance()

    def test_theme_is_dark_and_no_toggle(self):
        from GTray import GTray_CFG_UI
        ui = GTray_CFG_UI()
        self.assertEqual(ui.current_theme, 'dark')
        self.assertFalse(hasattr(ui, 'btn_theme_toggle'))
        self.assertIn('#0F172A', ui.styleSheet().upper())
        ui.close()

    def test_initial_status_is_paused(self):
        from GTray import GTray_CFG_UI
        ui = GTray_CFG_UI()
        self.assertEqual(ui.lbl_status.text(), "● Pausado")
        self.assertFalse(ui.is_running)
        self.assertEqual(ui.btn_startstop.text(), "Iniciar Serviço")
        
        # Testar transição de status
        ui.stop_buttons_def()
        self.assertEqual(ui.lbl_status.text(), "● Ativo")
        self.assertTrue(ui.is_running)
        self.assertEqual(ui.btn_startstop.text(), "Pausar Serviço")

        ui.start_buttons_def()
        self.assertEqual(ui.lbl_status.text(), "● Pausado")
        self.assertFalse(ui.is_running)
        self.assertEqual(ui.btn_startstop.text(), "Iniciar Serviço")
        ui.close()

    def test_api_key_visibility_toggle(self):
        from GTray import GTray_CFG_UI
        ui = GTray_CFG_UI()
        self.assertEqual(ui.input_provider_key.echoMode(), QLineEdit.Password)
        ui.toggle_apikey_visibility()
        self.assertEqual(ui.input_provider_key.echoMode(), QLineEdit.Normal)
        ui.toggle_apikey_visibility()
        self.assertEqual(ui.input_provider_key.echoMode(), QLineEdit.Password)
        ui.close()

    def test_tabs_and_crud_workflow(self):
        from GTray import GTray_CFG_UI
        ui = GTray_CFG_UI()
        
        # 3 Abas devem existir
        self.assertEqual(ui.tabs.count(), 3)
        self.assertEqual(ui.tabs.tabText(0), "⚡ Atalhos")
        self.assertEqual(ui.tabs.tabText(1), "🤖 Modelos")
        self.assertEqual(ui.tabs.tabText(2), "🔑 Provedores")

        # 1. Salvar Provedor (nome e chave)
        ui.input_provider_name.setText('groq_test')
        ui.input_provider_key.setText('gsk-test-key')
        ui.save_provider(show_msg=False)
        self.assertIn('groq_test', ui.data_manager.get_providers())

        # 2. Salvar Modelo (nome e provedor)
        ui.input_model_name.setText('llama-test')
        ui.combo_model_provider.setCurrentText('groq_test')
        ui.save_model(show_msg=False)
        self.assertIn('llama-test', ui.data_manager.get_models())
        self.assertEqual(ui.data_manager.get_model_provider('llama-test'), 'groq_test')

        # 3. Salvar Atalho (nome, tecla, modelo, pre-comando, extract_blocks e comment_char)
        self.assertTrue(hasattr(ui, 'input_shortcut_system_prompt'))
        self.assertTrue(hasattr(ui, 'checkbox_extract_blocks'))
        self.assertTrue(hasattr(ui, 'input_comment_char'))
        self.assertTrue(hasattr(ui, 'checkbox_shortcut_enabled'))
        self.assertTrue(ui.checkbox_shortcut_enabled.isChecked())

        ui.input_shortcut_name.setText('custom_test')
        ui.lbl_shortcut_key.setText('Ctrl+Alt+T')
        ui.combo_shortcut_model.setCurrentText('llama-test')
        ui.input_shortcut_system_prompt.setPlainText("Instrução customizada de sistema")
        ui.checkbox_extract_blocks.setChecked(True)
        ui.input_comment_char.setText("//")
        ui.checkbox_shortcut_enabled.setChecked(False)
        ui.save_shortcut(show_msg=False)

        resolved = ui.data_manager.resolve_shortcut('custom_test')
        self.assertEqual(resolved['key'], 'Ctrl+Alt+T')
        self.assertEqual(resolved['model_name'], 'llama-test')
        self.assertEqual(resolved['provider_name'], 'groq_test')
        self.assertEqual(resolved['api_key'], 'gsk-test-key')
        self.assertEqual(resolved['system_prompt'], "Instrução customizada de sistema")
        self.assertTrue(resolved['extract_blocks'])
        self.assertEqual(resolved['comment_char'], "//")
        self.assertFalse(resolved['enabled'])

        # Testa seleção e novo atalho
        ui.new_shortcut()
        self.assertEqual(ui.input_shortcut_system_prompt.toPlainText(), "")
        self.assertFalse(ui.checkbox_extract_blocks.isChecked())
        self.assertEqual(ui.input_comment_char.text(), "#")
        self.assertTrue(ui.checkbox_shortcut_enabled.isChecked())

        ui.on_shortcut_selected('custom_test')
        self.assertEqual(ui.input_shortcut_system_prompt.toPlainText(), "Instrução customizada de sistema")
        self.assertTrue(ui.checkbox_extract_blocks.isChecked())
        self.assertEqual(ui.input_comment_char.text(), "//")
        self.assertFalse(ui.checkbox_shortcut_enabled.isChecked())
        
        # Limpa dados de teste
        ui.data_manager.delete_shortcut('custom_test')
        ui.data_manager.delete_model('llama-test')
        ui.data_manager.delete_provider('groq_test')
        ui.close()

    def test_flash_message_on_save_and_delete(self):
        from unittest.mock import patch
        from PyQt5.QtWidgets import QMessageBox
        from GTray import GTray_CFG_UI
        ui = GTray_CFG_UI()

        # Componente deve existir e começar invisível
        self.assertTrue(hasattr(ui, 'lbl_flash_message'))
        self.assertFalse(ui.lbl_flash_message.isVisible())

        # Teste chamada direta show_flash_message
        ui.show_flash_message("Operação com sucesso!", "success")
        self.assertTrue(ui.lbl_flash_message.isVisible())
        self.assertIn("Operação com sucesso!", ui.lbl_flash_message.text())

        # Teste Salvar Atalho exibe flash message
        ui.input_shortcut_name.setText('flash_test_sc')
        ui.lbl_shortcut_key.setText('F12')
        ui.combo_shortcut_model.setCurrentText('gpt-4o-mini')
        ui.save_shortcut(show_msg=True)
        self.assertTrue(ui.lbl_flash_message.isVisible())
        self.assertIn("salvo", ui.lbl_flash_message.text().lower())

        # Teste Excluir Atalho exibe flash message
        with patch.object(QMessageBox, 'question', return_value=QMessageBox.Yes):
            ui.delete_shortcut()
            self.assertTrue(ui.lbl_flash_message.isVisible())
            self.assertIn("excluído", ui.lbl_flash_message.text().lower())

        # Limpeza
        ui.data_manager.delete_shortcut('flash_test_sc')
        ui.close()

    def test_buttons_click_shows_friendly_feedback(self):
        from GTray import GTray_CFG_UI
        ui = GTray_CFG_UI()

        # 1. Salvar Atalho - Fracasso (nome vazio via clique do botão)
        ui.input_shortcut_name.clear()
        ui.btn_save_shortcut.click()
        self.assertTrue(ui.lbl_flash_message.isVisible())
        self.assertIn("nome", ui.lbl_flash_message.text().lower())

        # 2. Salvar Atalho - Sucesso via clique do botão
        ui.input_shortcut_name.setText('btn_sc_test')
        ui.lbl_shortcut_key.setText('F8')
        ui.combo_shortcut_model.setCurrentText('gpt-4o-mini')
        ui.btn_save_shortcut.click()
        self.assertTrue(ui.lbl_flash_message.isVisible())
        self.assertIn("salvo", ui.lbl_flash_message.text().lower())

        # 3. Salvar Modelo - Fracasso (nome vazio via clique do botão)
        ui.input_model_name.clear()
        ui.btn_save_model.click()
        self.assertTrue(ui.lbl_flash_message.isVisible())
        self.assertIn("nome", ui.lbl_flash_message.text().lower())

        # 4. Salvar Modelo - Fracasso (sem provedor selecionado)
        ui.input_model_name.setText('btn_model_test')
        ui.combo_model_provider.setCurrentIndex(-1)
        ui.btn_save_model.click()
        self.assertTrue(ui.lbl_flash_message.isVisible())
        self.assertIn("provedor", ui.lbl_flash_message.text().lower())

        # 5. Salvar Modelo - Sucesso via clique do botão
        available_providers = list(ui.data_manager.get_providers().keys())
        prov_name = available_providers[0] if available_providers else 'openai'
        ui.combo_model_provider.setCurrentText(prov_name)
        ui.btn_save_model.click()
        self.assertTrue(ui.lbl_flash_message.isVisible())
        self.assertIn("salvo", ui.lbl_flash_message.text().lower())

        # 6. Salvar Provedor - Fracasso via clique do botão (nome vazio)
        ui.input_provider_name.clear()
        ui.btn_save_provider.click()
        self.assertTrue(ui.lbl_flash_message.isVisible())
        self.assertIn("nome", ui.lbl_flash_message.text().lower())

        # 7. Salvar Provedor - Sucesso via clique do botão
        ui.input_provider_name.setText('btn_prov_test')
        ui.input_provider_key.setText('test-key-123')
        ui.btn_save_provider.click()
        self.assertTrue(ui.lbl_flash_message.isVisible())
        self.assertIn("salvo", ui.lbl_flash_message.text().lower())

        # 8. Teste de Falha por Exceção (Atalho, Modelo e Provedor)
        with patch.object(ui.data_manager, 'set_shortcut', side_effect=RuntimeError("Disk full")):
            ui.btn_save_shortcut.click()
            self.assertTrue(ui.lbl_flash_message.isVisible())
            self.assertIn("falha", ui.lbl_flash_message.text().lower())

        with patch.object(ui.data_manager, 'set_model', side_effect=RuntimeError("Disk full")):
            ui.btn_save_model.click()
            self.assertTrue(ui.lbl_flash_message.isVisible())
            self.assertIn("falha", ui.lbl_flash_message.text().lower())

        with patch.object(ui.data_manager, 'set_provider', side_effect=RuntimeError("Disk full")):
            ui.btn_save_provider.click()
            self.assertTrue(ui.lbl_flash_message.isVisible())
            self.assertIn("falha", ui.lbl_flash_message.text().lower())

        # Limpeza
        ui.data_manager.delete_shortcut('btn_sc_test')
        ui.data_manager.delete_model('btn_model_test')
        ui.data_manager.delete_provider('btn_prov_test')
        ui.close()

    def test_listen_shortcut_key_displays_f8(self):
        from GTray import GTray_CFG_UI
        from services.klisten import KListen
        ui = GTray_CFG_UI()
        
        # Garante que começa com (nenhuma)
        ui.lbl_shortcut_key.setText("(nenhuma)")

        # Simula o retorno de KListen.run() capturando 'F8'
        with patch.object(KListen, 'run', return_value='F8'):
            ui.btn_listen_key.click()
            self.assertEqual(ui.lbl_shortcut_key.text(), 'F8')
            self.assertNotEqual(ui.lbl_shortcut_key.text(), '(nenhuma)')

        ui.close()

    def test_klisten_records_f8_and_combinations(self):
        import keyboard
        from services.klisten import KListen, format_key_combination

        kl = KListen(combo="test_f8")
        self.assertEqual(format_key_combination(['f8']), 'F8')
        self.assertEqual(format_key_combination(['ctrl', 'f8']), 'Ctrl+F8')

        # Teste evento F8
        ev_f8 = keyboard.KeyboardEvent('down', 66, 'f8')
        kl.record_key(ev_f8)
        self.assertEqual(kl.captured_key, 'F8')
        self.assertTrue(kl.flag_sair)

        # Teste unhook e reset não perdem captured_key pós run
        kl.stop_listening()
        self.assertEqual(kl.captured_key, 'F8')


if __name__ == '__main__':
    unittest.main()

