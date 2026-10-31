"""
pessoal/Engineer.py - Módulo de Automação de Área de Transferência e Teclado.

Responsável por capturar o texto atualmente selecionado no aplicativo em foco
(pré-processamento), injetar feedback visual transitório ao usuário e colar a
resposta final processada pela Inteligência Artificial (pós-processamento).
"""

import pyperclip
from pynput.keyboard import Key, Controller
from time import sleep
from pessoal.config_manager import ConfigManager

keyboard = Controller()

class Engineer:
    """Gerencia a captura da seleção de texto e inserção da resposta via clipboard."""

    def __init__(self, logger_from):
        """Inicializa o Engineer com o logger fornecido e o gerenciador de configurações."""
        global logger
        logger = logger_from
        self.config = ConfigManager()
        
    def pre_processamento(self) -> str:
        """
        Captura o texto selecionado na janela ativa simulando Ctrl+C.
        
        Substitui temporariamente o texto selecionado por 'Processando' para dar
        feedback visual imediato ao usuário enquanto a IA realiza o processamento.
        Retorna a string capturada ou vazia caso nada válido seja selecionado.
        """
        dados_clipboard = ""
        retry_clip = self.config.get_clip_retry()

        while dados_clipboard == "" and retry_clip > 0:
            retry_clip -= 1
            with keyboard.pressed(Key.ctrl):
                keyboard.press('c')
                keyboard.release('c')
            sleep(0.1)
            dados_clipboard = pyperclip.paste()
            sleep(0.1)

        if len(dados_clipboard) <= 1:
            logger.info('ignorado')
            with keyboard.pressed(Key.ctrl):
                keyboard.press('z')
                keyboard.release('z')
            return ""

        input = dados_clipboard
        retry_clip = self.config.get_clip_retry()

        while dados_clipboard != "Processando" and retry_clip > 0:
            retry_clip -= 1
            pyperclip.copy("Processando")
            sleep(0.1)
            dados_clipboard = pyperclip.paste()
            sleep(0.1)

        with keyboard.pressed(Key.ctrl):
            keyboard.press('v')
            keyboard.release('v')
        sleep(0.2)

        with keyboard.pressed(Key.ctrl):
            with keyboard.pressed(Key.shift):
                keyboard.press(Key.left)
                keyboard.release(Key.left)
        sleep(0.1)

        logger.info(f"Prompt: {input}")
        return input
    
    @staticmethod
    def pos_processamento(clipboard: str, response: str) -> None:
        """
        Copia a resposta gerada pela IA para a área de transferência e simula Ctrl+V
        no campo em foco para colar o resultado no lugar do texto original.
        """
        pyperclip.copy(response)
        sleep(0.1)
        with keyboard.pressed(Key.ctrl):
            keyboard.press('v')
            keyboard.release('v')
        
        logger.info('resposta entregue')

