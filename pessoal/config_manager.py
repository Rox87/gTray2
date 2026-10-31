"""
pessoal/config_manager.py - Gerenciador Central de Configurações da Aplicação.

Fornece acesso aos parâmetros globais da aplicação persistidos em `assets/gtray_data.json`,
como caminhos de ícones, tempos de retentativa, arquivos de log e configurações de IA,
descontinuando o arquivo legado assets/config.ini.
"""

from typing import Any, Optional
from gtray_data_manager import GTrayDataManager


class ConfigManager:
    """Gerenciador Singleton que lê as configurações a partir do gtray_data.json."""
    _instance = None
    _dm = None

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super(ConfigManager, cls).__new__(cls)
            cls._instance._initialize()
        return cls._instance

    def _initialize(self):
        self._dm = GTrayDataManager(sync_legacy=False)

    def get(self, section: str, key: str, fallback: Any = "") -> str:
        """
        Retorna valor de configuração buscando em 'app' ou retornando o fallback padrão.
        Mantém compatibilidade com chamadas get('GTRAY', '...').
        """
        key_lower = key.lower()
        defaults = {
            'favicon': 'assets/favicon.png',
            'gton': 'assets/favicon.png',
            'gtoff': 'assets/favicon.png',
            'header_title': 'gTray',
            'title': 'gTray',
            'lbl_shortcut_state_rec': 'Pressione uma combinação do teclado',
            'log': 'logs/app.log',
            'retry_clip': '140',
            'retry_ia': '14',
            'tema': 'dark'
        }
        val = self._dm.get_app_setting(key_lower, defaults.get(key_lower, fallback))
        return str(val) if val is not None else str(fallback)

    def get_int(self, section: str, key: str) -> int:
        """Retorna um valor de configuração como inteiro."""
        try:
            return int(self.get(section, key))
        except (ValueError, TypeError):
            if 'clip' in key.lower():
                return 140
            return 14

    def get_shortcut(self, shortcut_type: str) -> str:
        """Retorna a combinação de tecla do atalho a partir do gtray_data.json."""
        try:
            sc = self._dm.get_shortcut(shortcut_type)
            if sc and isinstance(sc, dict) and sc.get("key"):
                return sc.get("key")
        except Exception:
            pass
        return ""

    def get_model(self, shortcut_type: Optional[str] = None) -> str:
        """Retorna o modelo de IA associado ao atalho ou o primeiro modelo disponível."""
        if shortcut_type:
            sc = self._dm.get_shortcut(shortcut_type)
            if sc and isinstance(sc, dict) and sc.get("model"):
                return sc.get("model")
        models = self._dm.get_models()
        if models:
            return list(models.keys())[0]
        return "gpt-4o-mini"

    def get_provider(self, shortcut_type: Optional[str] = None) -> str:
        """Retorna o provedor associado ao modelo do atalho ou o primeiro disponível."""
        if shortcut_type:
            model = self.get_model(shortcut_type)
            prov = self._dm.get_model_provider(model)
            if prov:
                return prov
        providers = self._dm.get_providers()
        if providers:
            return list(providers.keys())[0]
        return "openai"

    def get_log_file(self) -> str:
        """Retorna o caminho do arquivo de log."""
        return self._dm.get_app_setting('log', 'logs/app.log')

    def get_retry_attempts(self) -> int:
        """Retorna número de tentativas para chamada de IA."""
        return int(self._dm.get_app_setting('retry_ia', 14))

    def get_clip_retry(self) -> int:
        """Retorna número de tentativas para operações na área de transferência."""
        return int(self._dm.get_app_setting('retry_clip', 140))