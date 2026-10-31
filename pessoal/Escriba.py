"""
pessoal/Escriba.py - Módulo de Registro de Logs do gTray.

Fornece configuração centralizada, profissional e estruturada de logs
com suporte a rotação de arquivos UTF-8, formatação padronizada e visualização
clara de atalhos e eventos da aplicação.
"""

import logging
from logging.handlers import RotatingFileHandler
import os
from typing import Dict, Any, Optional

DEFAULT_LOG_FORMAT = '%(asctime)s [%(levelname)-7s] [%(name)s] %(message)s'
DEFAULT_DATE_FORMAT = '%Y-%m-%d %H:%M:%S'

class registrador:
    """Gerenciador profissional de logs do gTray."""

    def __init__(self, logger_name: str = 'gTray'):
        self.logger_name = logger_name
        self.logger = logging.getLogger(logger_name)
        self.logger.setLevel(logging.DEBUG)

        # Garante diretório de logs
        logs_dir = os.path.join(os.path.dirname(__file__), '..', 'logs')
        os.makedirs(logs_dir, exist_ok=True)
        log_file = os.path.join(logs_dir, 'application.log')

        # Formatter profissional com data/hora padronizada e nível alinhado
        formatter = logging.Formatter(DEFAULT_LOG_FORMAT, datefmt=DEFAULT_DATE_FORMAT)

        # File Handler (Rotating) - UTF-8 explícito para caracteres especiais em PT-BR
        fh = RotatingFileHandler(
            log_file,
            maxBytes=10 * 1024 * 1024,  # 10 MB
            backupCount=5,
            encoding='utf-8',
            delay=True
        )
        fh.setLevel(logging.DEBUG)
        fh.setFormatter(formatter)

        # Console Stream Handler
        ch = logging.StreamHandler()
        ch.setLevel(logging.INFO)
        ch.setFormatter(formatter)

        # Limpeza preventiva e segura de handlers pré-existentes
        if self.logger.handlers:
            for handler in self.logger.handlers[:]:
                try:
                    handler.close()
                except Exception:
                    pass
                self.logger.removeHandler(handler)

        self.logger.addHandler(fh)
        self.logger.addHandler(ch)

        # Vincula referências úteis diretamente ao logger
        self.logger.registrador = self
        self.logger.log_shortcuts = self.log_shortcuts

    def addHand(self):
        """Método de retrocompatibilidade para reativação de handlers."""
        pass

    def get_logger(self) -> logging.Logger:
        """Retorna a instância do logger configurado."""
        return self.logger

    def log_shortcuts(self, shortcuts: Optional[Dict[str, Any]] = None, active_shortcut: Optional[str] = None):
        """
        Registra no log todos os atalhos cadastrados de forma limpa,
        estruturada e profissional, acompanhados da orientação de uso.
        """
        if shortcuts is None:
            try:
                from gtray_data_manager import GTrayDataManager
                dm = GTrayDataManager()
                shortcuts = dm.get_shortcuts()
            except Exception:
                shortcuts = {}

        self.logger.info("Atalhos cadastrados no sistema:")
        if isinstance(shortcuts, dict) and shortcuts:
            for name, data in shortcuts.items():
                if isinstance(data, dict):
                    key = data.get("key", "(nenhuma)")
                    model = data.get("model", "")
                    details = []
                    enabled = data.get("enabled", True)
                    status_text = "Ativo" if enabled else "Inativo"
                    details.append(f"Status: {status_text}")
                    if model:
                        details.append(f"Modelo: {model}")
                    if data.get("extract_blocks", False):
                        details.append("Extração de blocos: Ativa")
                    extra_str = f" ({' | '.join(details)})"
                    self.logger.info(f"  • {name}: {key}{extra_str}")
                else:
                    self.logger.info(f"  • {name}: {data}")
        else:
            self.logger.info("  (Nenhum atalho cadastrado no momento)")


    def log_header(self, title: str):
        """Registra um separador de seção formal no log."""
        border = "═" * 60
        self.logger.info(border)
        self.logger.info(f"  {title}")
        self.logger.info(border)

    # Métodos de conveniência
    def info(self, msg: str, *args, **kwargs):
        self.logger.info(msg, *args, **kwargs)

    def debug(self, msg: str, *args, **kwargs):
        self.logger.debug(msg, *args, **kwargs)

    def warning(self, msg: str, *args, **kwargs):
        self.logger.warning(msg, *args, **kwargs)

    def error(self, msg: str, *args, **kwargs):
        self.logger.error(msg, *args, **kwargs)

    def __getattr__(self, item):
        return getattr(self.logger, item)
