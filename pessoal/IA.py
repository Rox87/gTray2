"""
pessoal/IA.py - Camada de Integração Unificada com Modelos de Linguagem (LLMs).

Fornece uma interface agnóstica para envio de prompts e geração de respostas
utilizando a biblioteca oficial OpenAI compatível com múltiplos provedores
(OpenAI, DeepSeek, Groq, Ollama, OpenRouter, Together, Mistral, Gemini, etc.).
"""

import os
import re
from openai import OpenAI, AuthenticationError, PermissionDeniedError, NotFoundError, BadRequestError
from pessoal.Escriba import registrador
from pessoal.config_manager import ConfigManager

class IA:
    """Cliente unificado de Inteligência Artificial com suporte a chave dinâmica e endpoint base."""

    def __init__(self, key: str, logger_from, base_url: str = None):
        """
        Inicializa a instância de IA.

        :param key: Chave de API do provedor (Bearer Token).
        :param logger_from: Instância de logger para registro de eventos.
        :param base_url: URL base opcional para provedores compatíveis com a API OpenAI.
        """
        global logger
        logger = logger_from
        self.config = ConfigManager()
        self.key = key or ""
        self.base_url = base_url
        self._init_client()

    def _init_client(self):
        """Instancia o cliente OpenAI com a chave e a base_url configuradas."""
        kwargs = {"api_key": self.key if self.key else "dummy_key"}
        if self.base_url:
            kwargs["base_url"] = self.base_url
        self.client = OpenAI(**kwargs)

    def change_key(self, key: str, base_url: str = None):
        """
        Atualiza dinamicamente a chave de API e/ou a URL base sem reiniciar a aplicação.
        """
        changed = False
        if key is not None and key != self.key:
            self.key = key
            changed = True
        if base_url != self.base_url:
            self.base_url = base_url
            changed = True
        if changed or not hasattr(self, 'client'):
            self._init_client()

    def python_responder(self, prompt: str, mode: str = 'python', model_override: str = None, comment_char: str = "#") -> str:
        """
        Gera soluções em código Python com formatação estruturada de múltiplos arquivos.

        Detecta arquivos referenciados no formato @@caminho/arquivo@@, anexa seus conteúdos
        ao prompt, e formata a resposta comentando as linhas do prompt original e os cabeçalhos
        de arquivos com o caractere de comentário configurado.
        """
        def detectar_caminho_arquivo(texto):
            padrao = r'@@([^@]+[\\/][^@]+)@'
            return re.findall(padrao, texto)

        caminhos_arquivos = detectar_caminho_arquivo(prompt)
        final_prompt = prompt

        for caminho in caminhos_arquivos:
            try:
                logger.info(f"Reading file: {caminho}")
                with open(caminho, 'r', encoding='utf-8') as file:
                    final_prompt += f'\n#{caminho}\n' + file.read()
            except Exception as e:
                logger.error(f"Error reading file {caminho}: {str(e)}")

        target_model = model_override or self.config.get_model(mode)
        try:
            response = self.client.chat.completions.create(
                model=target_model,
                messages=[{"role": "system", "content": "seja extremamente breve e retorne apenas o nome entre ** seguido do bloco código python para cada arquivo da solução a seguir: " + final_prompt}]
            )
            coms = response.choices[0].message.content

            comment = (comment_char or "#").strip()
            if not comment:
                comment = "#"

            prompt_lines = [f"{comment} {line}".rstrip() for line in final_prompt.strip().splitlines()]
            final_code = "\n".join(prompt_lines) + "\n" if prompt_lines else ""

            padrao = r'\*\*(.*?)\*\*\n```(.*?)\n(.*?)```'
            matches = re.findall(padrao, coms, re.DOTALL)

            for match in matches:
                filename, lang, bloco = match
                logger.info(f"Generated code for: {filename}")
                final_code += f'{comment} {lang.strip()} - {filename.strip()}\n{bloco.strip()}\n'

            if not matches:
                code_pattern = r'```([a-zA-Z0-9_\-\+\.]*)\n(.*?)```'
                code_matches = re.findall(code_pattern, coms, re.DOTALL)
                for lang, bloco in code_matches:
                    lang_label = lang.strip() or "python"
                    final_code += f'{comment} {lang_label} - script.py\n{bloco.strip()}\n'
                if not code_matches:
                    final_code += coms

            return final_code.rstrip()

        except (AuthenticationError, PermissionDeniedError) as auth_err:
            logger.error(f"AI authentication error (verifique chave/provedor): {str(auth_err)}")
            raise
        except (BadRequestError, NotFoundError) as req_err:
            logger.error(f"AI request error (verifique modelo '{target_model}'): {str(req_err)}")
            raise
        except Exception as e:
            logger.error(f"Error in AI response: {str(e)}")
            raise

    def responder(self, prompt: str, cmd: str = "", mode: str = None, model_override: str = None) -> str:
        """
        Envia uma requisição de chat completions para a IA com suporte a retentativas.

        :param prompt: Mensagem do usuário capturada da seleção.
        :param cmd: Pré-comando / Mensagem do Sistema (system prompt) configurada para o atalho.
        :param mode: Nome do modo/atalho para fallback de modelo.
        :param model_override: Nome do modelo específico configurado para o atalho.
        :return: Resposta gerada pela IA.
        """
        messages = []
        if cmd and str(cmd).strip():
            messages.append({"role": "system", "content": str(cmd).strip()})
        messages.append({"role": "user", "content": prompt})
        response = None
        retry = self.config.get_retry_attempts()
        target_model = model_override or self.config.get_model(mode)

        while response is None and retry > 0:
            retry -= 1
            try:
                response = self.client.chat.completions.create(
                    model=target_model,
                    messages=messages
                )
                break
            except (AuthenticationError, PermissionDeniedError) as auth_err:
                logger.error(f"AI authentication error (verifique a chave de API e provedor): {str(auth_err)}")
                raise
            except (BadRequestError, NotFoundError) as req_err:
                logger.error(f"AI request error (verifique o modelo '{target_model}'): {str(req_err)}")
                raise
            except Exception as e:
                logger.error(f"AI request failed (attempts left: {retry}): {str(e)}")
                if retry == 0:
                    raise

        if not response or not response.choices:
            raise RuntimeError("Nenhuma resposta recebida da IA.")

        resposta = response.choices[0].message.content
        logger.info(f'Response received: {resposta[:100]}...')
        return resposta