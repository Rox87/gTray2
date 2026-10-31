import unittest
from unittest.mock import MagicMock, patch
from handler import Gtray
from pessoal.IA import IA

class TestSystemPromptIntegration(unittest.TestCase):
    @patch('handler.Image.open')
    @patch('handler.pystray.Icon')
    @patch('handler.win32gui.GetForegroundWindow', return_value=123)
    def test_handler_uses_system_prompt_from_shortcut(self, mock_fg, mock_icon, mock_img):
        gtray = Gtray(logger_from=MagicMock())
        gtray.Engineer = MagicMock()
        gtray.Engineer.pre_processamento.return_value = "Texto selecionado pelo usuario"
        gtray.AI = MagicMock()
        gtray.AI.responder.return_value = "Texto processado"

        # Mock resolve_shortcut com system_prompt customizado
        gtray.data_manager = MagicMock()
        gtray.data_manager.resolve_shortcut.return_value = {
            'api_key': 'test_key',
            'model_name': 'test_model',
            'base_url': None,
            'system_prompt': 'Traduza para espanhol'
        }

        gtray.handle_ai_request('custom_shortcut')

        gtray.AI.responder.assert_called_once_with(
            "Texto selecionado pelo usuario",
            "Traduza para espanhol",
            mode="custom_shortcut",
            model_override="test_model"
        )
        gtray.Engineer.pos_processamento.assert_called_once_with(
            "Texto selecionado pelo usuario",
            "Texto processado"
        )

    @patch('handler.Image.open')
    @patch('handler.pystray.Icon')
    @patch('handler.win32gui.GetForegroundWindow', return_value=123)
    def test_handler_falls_back_to_prompt_modifier_when_empty(self, mock_fg, mock_icon, mock_img):
        gtray = Gtray(logger_from=MagicMock())
        gtray.Engineer = MagicMock()
        gtray.Engineer.pre_processamento.return_value = "Texto"
        gtray.AI = MagicMock()
        gtray.AI.responder.return_value = "Resp"

        gtray.data_manager = MagicMock()
        gtray.data_manager.resolve_shortcut.return_value = {
            'api_key': 'key',
            'model_name': 'model',
            'base_url': None,
            'system_prompt': ''
        }

        gtray.handle_ai_request('upgrade', prompt_modifier="Modificador padrao")

        gtray.AI.responder.assert_called_once_with(
            "Texto",
            "Modificador padrao",
            mode="upgrade",
            model_override="model"
        )

    @patch('pessoal.IA.OpenAI')
    def test_ia_responder_builds_messages_correctly(self, mock_openai):
        mock_client = MagicMock()
        mock_openai.return_value = mock_client
        mock_resp = MagicMock()
        mock_resp.choices = [MagicMock()]
        mock_resp.choices[0].message.content = "Resposta da IA"
        mock_client.chat.completions.create.return_value = mock_resp

        logger_mock = MagicMock()
        ia = IA(key="test", logger_from=logger_mock)
        ia.client = mock_client

        # Chamada com system prompt
        res = ia.responder("Meu texto", cmd="Instrução do sistema", model_override="gpt-4o")
        self.assertEqual(res, "Resposta da IA")
        called_messages = mock_client.chat.completions.create.call_args[1]['messages']
        self.assertEqual(len(called_messages), 2)
        self.assertEqual(called_messages[0], {"role": "system", "content": "Instrução do sistema"})
        self.assertEqual(called_messages[1], {"role": "user", "content": "Meu texto"})

        # Chamada sem system prompt
        ia.responder("Meu texto", cmd="", model_override="gpt-4o")
        called_messages_empty = mock_client.chat.completions.create.call_args[1]['messages']
        self.assertEqual(len(called_messages_empty), 1)
        self.assertEqual(called_messages_empty[0], {"role": "user", "content": "Meu texto"})

    @patch('handler.Image.open')
    @patch('handler.pystray.Icon')
    @patch('handler.win32gui.GetForegroundWindow', return_value=123)
    def test_extract_code_blocks_functionality(self, mock_fg, mock_icon, mock_img):
        gtray = Gtray(logger_from=MagicMock())

        # Teste 1: Múltiplos blocos com nome de arquivo na primeira palavra acima
        ai_response = (
            "Aqui está o script principal:\n"
            "main.py\n"
            "```python\n"
            "print('hello main')\n"
            "```\n\n"
            "E aqui está o módulo de utilitários:\n"
            "utils.py:\n"
            "```python\n"
            "def add(a, b): return a + b\n"
            "```"
        )
        extracted = gtray.extract_code_blocks(ai_response, comment_char="#")
        expected = (
            "# main.py\n"
            "print('hello main')\n\n"
            "# utils.py\n"
            "def add(a, b): return a + b"
        )
        self.assertEqual(extracted.strip(), expected.strip())

        # Teste 2: Nome com marcação markdown (ex: **script.js**) e caractere de comentário //
        ai_response_js = (
            "Veja o arquivo abaixo:\n"
            "**app.js**\n"
            "```javascript\n"
            "console.log('running');\n"
            "```"
        )
        extracted_js = gtray.extract_code_blocks(ai_response_js, comment_char="//")
        expected_js = (
            "// app.js\n"
            "console.log('running');"
        )
        self.assertEqual(extracted_js.strip(), expected_js.strip())

        # Teste 3: Sem palavra/nome acima do bloco
        ai_response_plain = (
            "```python\n"
            "x = 42\n"
            "```"
        )
        extracted_plain = gtray.extract_code_blocks(ai_response_plain, comment_char="#")
        self.assertEqual(extracted_plain.strip(), "x = 42")

        # Teste 4: Sem nenhum bloco de código na resposta
        ai_response_no_code = "Apenas uma resposta em texto puro."
        extracted_no_code = gtray.extract_code_blocks(ai_response_no_code, comment_char="#")
        self.assertEqual(extracted_no_code, ai_response_no_code)

    @patch('handler.Image.open')
    @patch('handler.pystray.Icon')
    @patch('handler.win32gui.GetForegroundWindow', return_value=123)
    def test_handler_applies_extract_blocks_when_configured(self, mock_fg, mock_icon, mock_img):
        gtray = Gtray(logger_from=MagicMock())
        gtray.Engineer = MagicMock()
        gtray.Engineer.pre_processamento.return_value = "código aqui"
        gtray.AI = MagicMock()
        gtray.AI.responder.return_value = (
            "script.py\n"
            "```python\n"
            "print('extraido')\n"
            "```"
        )

        gtray.data_manager = MagicMock()
        gtray.data_manager.resolve_shortcut.return_value = {
            'api_key': 'k',
            'model_name': 'm',
            'base_url': None,
            'system_prompt': '',
            'extract_blocks': True,
            'comment_char': '#'
        }

        gtray.handle_ai_request('generic')
        gtray.Engineer.pos_processamento.assert_called_once_with(
            "código aqui",
            "# script.py\nprint('extraido')"
        )

    @patch('handler.Image.open')
    @patch('handler.pystray.Icon')
    @patch('handler.win32gui.GetForegroundWindow', return_value=123)
    def test_handler_passes_comment_char_to_python_responder(self, mock_fg, mock_icon, mock_img):
        gtray = Gtray(logger_from=MagicMock())
        gtray.Engineer = MagicMock()
        gtray.Engineer.pre_processamento.return_value = "prompt python"
        gtray.AI = MagicMock()
        gtray.AI.python_responder.return_value = "# prompt python\n# python - main.py\ncode"

        gtray.data_manager = MagicMock()
        gtray.data_manager.resolve_shortcut.return_value = {
            'api_key': 'k',
            'model_name': 'm',
            'base_url': None,
            'system_prompt': '',
            'extract_blocks': True,
            'comment_char': '//'
        }

        gtray.handle_ai_request('python')

        gtray.AI.python_responder.assert_called_once_with(
            "prompt python",
            mode="python",
            model_override="m",
            comment_char="//"
        )

    @patch('handler.Image.open')
    @patch('handler.pystray.Icon')
    @patch('handler.win32gui.GetForegroundWindow', return_value=123)
    def test_handler_returns_full_response_when_extract_blocks_disabled(self, mock_fg, mock_icon, mock_img):
        gtray = Gtray(logger_from=MagicMock())
        gtray.Engineer = MagicMock()
        gtray.Engineer.pre_processamento.return_value = "imprima de 1 a 10"
        gtray.AI = MagicMock()
        raw_ai_response = (
            "Aqui está o código solicitado:\n"
            "```python\n"
            "for i in range(1, 11):\n"
            "    print(i)\n"
            "```\n"
            "Qualquer dúvida estou à disposição!"
        )
        gtray.AI.responder.return_value = raw_ai_response

        # Modo Programação desmarcado (extract_blocks: False)
        gtray.data_manager = MagicMock()
        gtray.data_manager.resolve_shortcut.return_value = {
            'api_key': 'k',
            'model_name': 'm',
            'base_url': None,
            'system_prompt': 'seja breve',
            'extract_blocks': False,
            'comment_char': '#'
        }

        # Mesmo para modo 'python', se extract_blocks for False deve retornar integralmente
        gtray.handle_ai_request('python')

        gtray.AI.python_responder.assert_not_called()
        gtray.AI.responder.assert_called_once_with(
            "imprima de 1 a 10",
            "seja breve",
            mode="python",
            model_override="m"
        )
        gtray.Engineer.pos_processamento.assert_called_once_with(
            "imprima de 1 a 10",
            raw_ai_response
        )

    @patch('pessoal.IA.OpenAI')
    def test_python_responder_formats_with_comment_char(self, mock_openai):
        mock_client = MagicMock()
        mock_openai.return_value = mock_client
        mock_resp = MagicMock()
        mock_resp.choices = [MagicMock()]
        mock_resp.choices[0].message.content = "**main.py**\n```python\nfor i in range(1, 101):\n    print(i)\n```"
        mock_client.chat.completions.create.return_value = mock_resp

        logger_mock = MagicMock()
        ia = IA(key="test", logger_from=logger_mock)
        ia.client = mock_client

        res = ia.python_responder("Imprima os números de 1 a 100.", comment_char="#")
        expected = (
            "# Imprima os números de 1 a 100.\n"
            "# python - main.py\n"
            "for i in range(1, 101):\n"
            "    print(i)"
        )
        self.assertEqual(res.strip(), expected.strip())

    @patch('handler.keyboard.add_hotkey')
    @patch('handler.keyboard.remove_hotkey')
    @patch('handler.Image.open')
    @patch('handler.pystray.Icon')
    def test_register_shortcuts_respects_enabled_flag(self, mock_icon, mock_img, mock_rm, mock_add):
        gtray = Gtray(logger_from=MagicMock())
        gtray.config = MagicMock()
        gtray.config.get_shortcut.side_effect = lambda sc: 'F8' if sc == 'upgrade' else ('F9' if sc == 'generic' else '')
        gtray.data_manager = MagicMock()
        gtray.data_manager.get_shortcut.side_effect = lambda sc: {'enabled': True} if sc == 'upgrade' else {'enabled': False}

        gtray.register_shortcuts()

        added_hotkeys = [call[0][0] for call in mock_add.call_args_list]
        self.assertIn('f8', added_hotkeys)
        self.assertNotIn('f9', added_hotkeys)

if __name__ == '__main__':
    unittest.main()
