# Redesenho Visual do gTray: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transformar o visual do gTray em uma interface desktop minimalista, moderna e profissional em PyQt5, com suporte aos temas Dark (padrão) e Light, eliminando elementos pesados e desproporcionais.

**Architecture:** Módulo central de estilos `ui_theme.py` provendo tokens e folhas de estilo QSS para os temas 'dark' e 'light'. A classe `GTray_CFG_UI` em `GTray.py` é refatorada para organizar os controles em cards visuais limpos (Cabeçalho com toggle de tema e badge de status, Card API Key, Card Atalhos, Preferências e Ações no Rodapé). O tema escolhido é persistido em `assets/config.ini`.

**Tech Stack:** Python 3.x, PyQt5, configparser, unittest.

**Spec:** [docs/superpowers/specs/2026-10-12-gtray-visual-redesign-design.md](file:///c:/Users/rodri/gitSpace/gTray/docs/superpowers/specs/2026-10-12-gtray-visual-redesign-design.md)

## Global Constraints

- Sistema operacional Windows, interface gráfica construída estritamente em PyQt5.
- Tema padrão obrigatório: 'dark' (Opção A: Dark Mode Studio); tema alternativo: 'light' (Opção B: Modern Clean Light).
- Dimensões compactas da janela (~380x520 px) centralizada na tela.
- Manter 100% de compatibilidade com argumentos de linha de comando (`sys.argv[1]` = "1", "2" ou sem argumentos).
- Manter integridade funcional das chamadas à IA (`handler.py`) e captura de atalhos (`services/klisten.py`).
- Persistir o tema ativo em `assets/config.ini` sob a chave `tema`.

## Review Focus

1. **Configuração sem chave de tema:** Se `assets/config.ini` não contiver `tema`, o app deve carregar silenciosamente o tema 'dark' sem erros ou travas.
2. **QSS sem erros de parse:** A folha de estilo deve ser 100% válida em conformidade com o parser do PyQt5, sem gerar warnings como `Could not parse stylesheet of object`.
3. **Alternância dinâmica de tema:** Clicar no botão de alternar tema deve atualizar instantaneamente todos os componentes (fundo, textos, inputs, botões, bordas e ícones) sem necessidade de reiniciar.
4. **Modo de senha e revelação da API Key:** Ao abrir, a chave deve ser mascarada (`Password` echo mode); o botão `👁` deve alternar entre modo normal e senha sem expor acidentalmente o conteúdo no log ou bloquear a edição.
5. **Gravação de atalho com feedback claro:** O processo de gravação de atalho (`KListen`) deve atualizar visualmente o badge do atalho para o estado de gravação e restaurar o estado normal mesmo se cancelado via Esc.

---

### Task 1: Módulo de Design System e Temas (`ui_theme.py`)

**Files:**
- Create: `ui_theme.py`
- Test: `teste/test_ui_theme.py`

**Interfaces:**
- Produces:
  - `get_theme_stylesheet(theme_name: str) -> str`: Retorna a folha de estilo QSS completa para `dark` ou `light` (com fallback seguro para `dark`).
  - `get_status_badge_style(is_active: bool, theme_name: str) -> str`: Retorna estilo para o badge de status (Ativo / Pausado).
  - Constantes de paleta para os temas 'dark' e 'light'.

- [ ] **Step 1: Escrever teste unitário para `ui_theme.py`**

```python
# teste/test_ui_theme.py
import unittest
from ui_theme import get_theme_stylesheet, get_status_badge_style

class TestUITheme(unittest.TestCase):
    def test_dark_stylesheet_contains_palette(self):
        qss = get_theme_stylesheet('dark')
        self.assertIn('#0F172A', qss.upper())
        self.assertIn('QPushButton', qss)
        self.assertIn('QLineEdit', qss)

    def test_light_stylesheet_contains_palette(self):
        qss = get_theme_stylesheet('light')
        self.assertIn('#F8FAFC', qss.upper())
        self.assertIn('QPushButton', qss)

    def test_fallback_to_dark_for_unknown_theme(self):
        qss = get_theme_stylesheet('unknown_theme')
        self.assertEqual(qss, get_theme_stylesheet('dark'))

    def test_status_badge_styles(self):
        active_style = get_status_badge_style(True, 'dark')
        self.assertIn('#22C55E', active_style.upper())
        inactive_style = get_status_badge_style(False, 'dark')
        self.assertTrue('#94A3B8' in inactive_style.upper() or '#EF4444' in inactive_style.upper())
```

- [ ] **Step 2: Executar o teste para verificar falha (arquivo ainda não existe)**

Comando: `python -m unittest teste.test_ui_theme -v`
Resultado esperado: FAIL / ImportError: No module named 'ui_theme'

- [ ] **Step 3: Implementar `ui_theme.py`**

Criar `ui_theme.py` contendo:
- Definição dos tokens para tema Dark (`#0F172A` background, `#1E293B` cards, `#334155` borders, `#F8FAFC` text, `#94A3B8` muted, `#3B82F6` primary accent, `#22C55E` success, `#EF4444` danger).
- Definição dos tokens para tema Light (`#F8FAFC` background, `#FFFFFF` cards, `#E2E8F0` borders, `#0F172A` text, `#64748B` muted, `#2563EB` primary accent, `#16A34A` success, `#DC2626` danger).
- Regras QSS completas para `QMainWindow`, `QWidget`, `QFrame[card="true"]`, `QLabel`, `QLineEdit`, `QComboBox`, `QPushButton`, `QCheckBox`.
- Implementação de `get_theme_stylesheet(theme_name: str) -> str` e `get_status_badge_style(is_active: bool, theme_name: str) -> str`.

- [ ] **Step 4: Executar o teste para verificar aprovação**

Comando: `python -m unittest teste.test_ui_theme -v`
Resultado esperado: PASS (todos os 4 testes passam).

---

### Task 2: Persistência de Tema no `assets/config.ini`

**Files:**
- Modify: `assets/config.ini`
- Test: `teste/test_config_theme.py`
- Modify: `GTray.py` (métodos de leitura/gravação de configuração de tema)

**Interfaces:**
- Consumes: `assets/config.ini`
- Produces:
  - Leitura segura de `tema` no `assets/config.ini` com fallback para `dark`.
  - Método `save_theme(theme_name: str)` para persistir no arquivo INI sem perder outras seções.

- [ ] **Step 1: Escrever teste unitário para persistência de tema**

```python
# teste/test_config_theme.py
import unittest
import configparser
import os
import tempfile

class TestConfigTheme(unittest.TestCase):
    def test_theme_fallback_when_absent(self):
        cp = configparser.ConfigParser()
        cp.read_string("[GTRAY]\nheader_title=GTray\n")
        theme = cp.get('GTRAY', 'tema', fallback='dark')
        self.assertEqual(theme, 'dark')

    def test_theme_read_and_write(self):
        with tempfile.NamedTemporaryFile('w+', delete=False, suffix='.ini', encoding='utf-8') as f:
            f.write("[GTRAY]\nheader_title=GTray\ntema=light\n")
            temp_path = f.name
        try:
            cp = configparser.ConfigParser()
            cp.read(temp_path, encoding='utf-8')
            self.assertEqual(cp.get('GTRAY', 'tema'), 'light')
            
            cp.set('GTRAY', 'tema', 'dark')
            with open(temp_path, 'w', encoding='utf-8') as f:
                cp.write(f)
                
            cp2 = configparser.ConfigParser()
            cp2.read(temp_path, encoding='utf-8')
            self.assertEqual(cp2.get('GTRAY', 'tema'), 'dark')
        finally:
            os.remove(temp_path)
```

- [ ] **Step 2: Executar teste de persistência para verificar comportamento esperado**

Comando: `python -m unittest teste.test_config_theme -v`
Resultado esperado: PASS

- [ ] **Step 3: Adicionar `tema=dark` em `assets/config.ini`**

Adicionar sob `[GTRAY]`:
`tema=dark`

---

### Task 3: Refatoração da Janela Principal em `GTray.py`

**Files:**
- Modify: `GTray.py`
- Test: `teste/test_gtray_ui.py`

**Interfaces:**
- Consumes:
  - `ui_theme.get_theme_stylesheet`
  - `ui_theme.get_status_badge_style`
  - `assets/config.ini`
- Modifica:
  - `GTray_CFG_UI.initUI()`
  - Adiciona `toggle_theme()`, `set_theme(theme_name: str)`
  - Refatora `input_blockunblock()` e `toggle_password_visibility()`
  - Moderniza layout em cards (`header_frame`, `apikey_frame`, `shortcut_frame`, `footer_frame`)
  - Substitui estilos antigos inline por classes e QSS limpo

- [ ] **Step 1: Escrever teste unitário para inicialização da UI e troca de tema**

```python
# teste/test_gtray_ui.py
import unittest
import sys
from PyQt5.QtWidgets import QApplication
from PyQt5.QtCore import Qt

class TestGTrayUI(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if not QApplication.instance():
            cls.app = QApplication(sys.argv)
        else:
            cls.app = QApplication.instance()

    def test_theme_switch_updates_stylesheet(self):
        from GTray import GTray_CFG_UI
        ui = GTray_CFG_UI()
        self.assertEqual(ui.current_theme, 'dark')
        ui.toggle_theme()
        self.assertEqual(ui.current_theme, 'light')
        ui.toggle_theme()
        self.assertEqual(ui.current_theme, 'dark')
        ui.close()

    def test_api_key_visibility_toggle(self):
        from GTray import GTray_CFG_UI
        from PyQt5.QtWidgets import QLineEdit
        ui = GTray_CFG_UI()
        self.assertEqual(ui.input_apikey.echoMode(), QLineEdit.Password)
        ui.toggle_apikey_visibility()
        self.assertEqual(ui.input_apikey.echoMode(), QLineEdit.Normal)
        ui.toggle_apikey_visibility()
        self.assertEqual(ui.input_apikey.echoMode(), QLineEdit.Password)
        ui.close()
```

- [ ] **Step 2: Executar teste para verificar falha nos métodos novos**

Comando: `python -m unittest teste.test_gtray_ui -v`
Resultado esperado: FAIL (atributos `current_theme`, `toggle_theme`, `toggle_apikey_visibility` ainda não implementados em `GTray_CFG_UI`).

- [ ] **Step 3: Refatorar `GTray.py` com o novo design moderno**

1. Importar `ui_theme` (`from ui_theme import get_theme_stylesheet, get_status_badge_style`).
2. Ler `tema` de `config.ini` em `__init__`, guardando em `self.current_theme = config.get('GTRAY', 'tema', fallback='dark')`.
3. Reorganizar `initUI()`:
   - Configurar janela: `self.setFixedSize(380, 520)`, `self.setWindowTitle('gTray')`.
   - Aplicar `self.setStyleSheet(get_theme_stylesheet(self.current_theme))`.
   - **Header Frame:** Logo compacto de 28x28, título "gTray" em 16px bold, badge de status (`self.lbl_status`), botão de alternância de tema (`self.btn_theme_toggle` com ☀️ ou 🌙).
   - **API Key Frame:** Título da seção em maiúsculas suaves, `self.input_apikey` mascarado (`QLineEdit.Password`), botão de visualização (`self.btn_show_key`), botão de edição/salvar (`self.btn_apikey`), botão de ajuda (`self.btn_help_key`).
   - **Shortcuts Frame:** Título da seção, `self.shortcut_type_combo` estilizado, badge elegante para `self.lbl_shortcut` exibindo a combinação com destaque monoespaçado, e botão `self.btn_shortcut` ("Gravar Atalho").
   - **Preferences Row:** Checkbox "Iniciar com o Windows" e "Manter Visível" alinhados com espaçamento uniforme.
   - **Footer Action Frame:** Botão principal `self.btn_startstop` ("Iniciar Serviço" / "Pausar Serviço") e botão "Minimizar" (`self.btn_minimize`).
4. Implementar `toggle_theme()`:
   - Alterna entre `'dark'` e `'light'`, chama `set_theme(new_theme)`.
   - Atualiza `self.setStyleSheet(get_theme_stylesheet(self.current_theme))`.
   - Atualiza ícone do botão (`☀️` quando dark, `🌙` quando light).
   - Salva em `assets/config.ini`.
5. Implementar `toggle_apikey_visibility()`:
   - Alterna `echoMode` entre `QLineEdit.Password` e `QLineEdit.Normal`.
6. Refatorar `start_buttons_def` e `stop_buttons_def` para atualizar texto e badge de status sem quebrar folhas de estilo QSS.
7. Corrigir formatação dos logs e evitar erros residuais no console.

- [ ] **Step 4: Executar o teste para verificar aprovação**

Comando: `python -m unittest teste.test_gtray_ui -v`
Resultado esperado: PASS (todos os testes passam).

---

### Task 4: Validação Funcional Completa e Limpeza

**Files:**
- Verify: `GTray.py`
- Modify: `.gitignore` (adicionar `.superpowers/` para não poluir o repositório)

- [ ] **Step 1: Adicionar `.superpowers/` ao `.gitignore`**

Garantir que os arquivos temporários do brainstorm fiquem ignorados no git.

- [ ] **Step 2: Executar suite completa de testes**

Comando: `python -m unittest discover -s teste -p "test_*.py" -v`
Resultado esperado: Todos os testes executados com sucesso (0 falhas).

- [ ] **Step 3: Teste de execução rápida de inicialização**

Comando: `python -c "from GTray import GTray_CFG_UI; print('GTray_CFG_UI carregado com sucesso!')"`
Resultado esperado: Saída limpa sem avisos de stylesheet do Qt.
