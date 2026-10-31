# gTray ⚡

> **Assistente de Inteligência Artificial para Windows integrado via Atalhos Globais e Bandeja do Sistema.**

O **gTray** é uma ferramenta de produtividade desenvolvida em Python para Windows que permite utilizar modelos de Inteligência Artificial em qualquer aplicativo do seu computador (editores de código, navegadores, suítes office, mensageiros, terminais, etc.). 

Basta selecionar um texto em qualquer campo editável e acionar uma combinação de teclas: o gTray captura a seleção, processa a solicitação através da IA configurada e cola a resposta no mesmo local instantaneamente.

---

## 📑 Sumário

- [Visão Geral e Fluxo de Funcionamento](#-visão-geral-e-fluxo-de-funcionamento)
- [Principais Funcionalidades](#-principais-funcionalidades)
- [Atalhos e Modos Pré-configurados](#-atalhos-e-modos-pré-configurados)
- [Arquitetura do Projeto](#-arquitetura-do-projeto)
- [Pré-requisitos e Instalação](#-pré-requisitos-e-instalação)
- [Guia Rápido de Configuração](#-guia-rápido-de-configuração)
- [Execução dos Testes Automatizados](#-execução-dos-testes-automatizados)
- [Segurança e Privacidade](#-segurança-e-privacidade)
- [Licença](#-licença)

---

## 🔄 Visão Geral e Fluxo de Funcionamento

O gTray atua em segundo plano de forma não intrusiva. Quando um atalho ativo é pressionado, o seguinte fluxo é executado:

```text
[ Qualquer Aplicativo do Windows ]
       │
       │  1. Usuário seleciona o texto e pressiona a tecla de atalho
       ▼
 [ handler.py ] ──▶ [ Engineer.py ] (Simula Ctrl+C e exibe "Processando...")
       │
       │  2. Resolução da cadeia: Atalho ➔ Modelo ➔ Provedor ➔ API Key & Base URL
       ▼
 [ IA.py ] ───────▶ [ Provedor LLM ] (OpenAI, DeepSeek, Groq, Ollama, etc.)
       │
       │  3. Formatação da resposta (Extração de blocos / Comentários se configurado)
       ▼
 [ Engineer.py ] ─▶ Cola o resultado in-place no aplicativo ativo (Simula Ctrl+V)
```

---

## ✨ Principais Funcionalidades

### 🌐 Suporte Amplo a Provedores (Multi-Provider)
- Suporte a qualquer serviço compatível com a API padrão OpenAI: **OpenAI**, **DeepSeek**, **Groq**, **Anthropic**, **OpenRouter**, **Together AI**, **Mistral**, **Google Gemini** e modelos locais/auto-hospedados via **Ollama** e **vLLM**.
- URLs base (`base_url`) detectadas automaticamente ou configuráveis manualmente por provedor.

### 🧠 Gerenciamento Flexível de Modelos
- Cadastre e gerencie múltiplos modelos de linguagem (ex: `gpt-4o-mini`, `deepseek-chat`, `llama-3.3-70b-versatile`).
- Vincule livremente cada modelo ao provedor correspondente.

### ⌨️ Atalhos 100% Personalizáveis
- **Gravação Interativa de Teclas**: Pressione o botão *Gravar Tecla* e acione a combinação desejada (ex: `F8`, `F9`, `Ctrl+Alt+T`).
- **Pré-comando / System Prompt Customizado**: Defina o papel e as instruções de sistema da IA individualmente para cada atalho (ex: *"Apenas reescreva o texto sem explicações"* ou *"Traduza para inglês"*).
- **Extração de Código (Modo Programação)**: Remove textos introdutórios e extrai apenas o código cercado por markdown, prefixando o nome do arquivo comentado.
- **Caractere de Comentário Configurável**: Defina o caractere adequado para cada contexto (`#`, `//`, `--`, etc.).
- **Ativação / Desativação Seletiva (Checkbox "Atalho ativo")**: Desative temporariamente um atalho sem precisar excluí-lo.
- **Hot-Reload em Tempo Real**: Alterações salvas na interface têm efeito imediato nas teclas registradas no sistema, sem precisar reiniciar a aplicação.

### 🎨 Interface Gráfica Moderna (PyQt5)
- Tema escuro (*Dark Mode*) com paleta profissional e design limpo.
- Sistema de **Mensagens Flash temporizadas** com feedback visual imediato no rodapé:
  - 🟢 **Verde**: Operações realizadas com sucesso.
  - 🟡 **Amarelo**: Alertas e avisos de preenchimento.
  - 🔴 **Vermelho**: Erros ou confirmações de exclusão.
- Navegação organizada em abas: **Atalhos**, **Modelos**, **Provedores** e **Configurações Gerais**.

### 🛡️ Execução Silenciosa na Bandeja do Sistema (*System Tray*)
- Minimiza silenciosamente para a bandeja via `pystray`.
- Menu de contexto rápido com opções de **Mostrar UI** e **Sair**.

### 📝 Logs Profissionais e Estruturados (`Escriba.py`)
- Logs com formatação padronizada (`data/hora`, `nível de log` e `nome do módulo`).
- Codificação explícita em **UTF-8** para compatibilidade total com caracteres em português.
- Rotação automática de arquivos de log (`RotatingFileHandler` de até 10 MB) para preservar o disco.
- Listagem detalhada no log ao iniciar a aplicação com todos os atalhos cadastrados, seus modelos, status (*Ativo* / *Inativo*) e flags.

---

## 📌 Atalhos e Modos Pré-configurados

| Modo | Tecla Padrão | Modelo Padrão | Descrição |
| :--- | :---: | :---: | :--- |
| **upgrade** | `F8` / `F9` | `gpt-4o-mini` | **Melhoria direta de texto**: Retorna diretamente o texto selecionado aprimorado, sem rodeios ou saudações. |
| **generic** | `F7+Ctrl+X` | `gpt-4o-mini` | **Assistente geral**: Envia a seleção para o modelo de IA e responde de forma abrangente. |
| **python** | `F10` | `gpt-4o-mini` | **Gerador de código Python**: Lê referências a arquivos no texto (`@@caminho/arquivo@@`), gera soluções comentadas e estrutura múltiplos arquivos. |
| **creation** | `F11` | `gpt-4o-mini` | **Assistente de projetos**: Abre a interface gráfica complementar para geração de estruturas de código e projetos. |

> 💡 *Todos os nomes, teclas, modelos e instruções de sistema acima podem ser alterados ou novos atalhos podem ser criados a qualquer momento pela interface.*

---

## 📂 Arquitetura do Projeto

```text
gTray/
├── GTray.py                  # Ponto de entrada e interface gráfica principal (PyQt5)
├── handler.py                # Controlador de eventos globais, bandeja do sistema e hotkeys
├── gtray_data_manager.py     # Camada de persistência JSON e resolução em cadeia de atalhos
├── ui_theme.py               # Design system e folha de estilos QSS (Dark Theme)
│
├── pessoal/                  # Módulos centrais de apoio e lógica de negócio
│   ├── Engineer.py           # Automação de clipboard e emulação de teclado
│   ├── Escriba.py            # Gerenciador estruturado de logs e rotação UTF-8
│   ├── IA.py                 # Integração agnóstica com clientes LLM compatíveis com OpenAI
│   ├── config_manager.py     # Gerenciador de configurações baseado em gtray_data.json
│   └── Decorador.py          # Decoradores de telemetria e tempo de resposta
│
├── assets/                   # Recursos visuais e arquivos de configuração
│   ├── gtray_data.json       # Base de dados única da aplicação (app, atalhos, modelos e provedores)
│   ├── favicon.png           # Ícone do aplicativo
│   └── ...
│
├── cmd/                      # Scripts auxiliares para execução em segundo plano
│   ├── run.bat               # Inicia o gTray em modo background
│   └── run.vbs               # Launcher sem janela de console
│
├── logs/                     # Diretório de saída dos registros de execução
│   └── application.log       # Arquivo de log rotativo gerado em tempo de execução
│
└── teste/                    # Suíte completa de testes unitários automatizados
    ├── test_gtray_ui.py
    ├── test_gtray_data_manager.py
    ├── test_system_prompt_integration.py
    ├── test_escriba.py
    └── ...
```

---

## 🚀 Pré-requisitos e Instalação

### Pré-requisitos
- **Sistema Operacional:** Windows 10 ou Windows 11.
- **Python:** Versão 3.10 ou superior.
- **Chave de API:** Uma chave de API válida de pelo menos um provedor de IA (OpenAI, DeepSeek, Groq, etc.) ou uma instância local do Ollama ativa.

### Passo a Passo de Instalação

1. **Clone o repositório:**
   ```bash
   git clone https://github.com/Rox87/gTray.git
   cd gTray
   ```

2. **Crie e ative um ambiente virtual (recomendado):**
   ```bash
   python -m venv venv
   .\venv\Scripts\activate
   ```

3. **Instale as dependências necessárias:**
   ```bash
   pip install -r requirements.txt
   ```

4. **Inicie o gTray:**
   ```bash
   python GTray.py
   ```

---

## 🎯 Guia Rápido de Configuração

### 1. Configurar um Provedor de IA
1. Abra a aplicação e selecione a aba **Provedores**.
2. Clique em **+ Novo** ou selecione um provedor existente (ex: `deepseek` ou `openai`).
3. Digite o nome do provedor, sua **Chave de API** e, se aplicável, a **Base URL** (para provedores conhecidos como DeepSeek e Groq, a URL é preenchida automaticamente).
4. Clique em **Salvar Provedor**.

### 2. Configurar um Modelo
1. Vá para a aba **Modelos**.
2. Digite o nome do modelo (ex: `deepseek-chat`, `gpt-4o-mini`, `llama-3.3-70b-versatile`).
3. Selecione o provedor correspondente no menu suspenso.
4. Clique em **Salvar Modelo**.

### 3. Configurar ou Criar um Atalho
1. Vá para a aba **Atalhos**.
2. Selecione um atalho existente ou clique em **+ Novo**.
3. Defina o **Nome do Atalho** e clique em **Gravar Tecla** para capturar a combinação do teclado.
4. Associe o **Modelo de IA** cadastrado.
5. *(Opcional)* Preencha a **Mensagem do Sistema** com as instruções específicas para este atalho.
6. *(Opcional)* Marque **Modo Programação** caso deseje extração limpa de código.
7. Garanta que o checkbox **Atalho ativo** esteja marcado.
8. Clique em **Salvar Atalho**.

### 4. Iniciar o Serviço
1. Clique no botão **Iniciar**.
2. O gTray registrará os atalhos no Windows e se integrará à bandeja do sistema.
3. Teste selecionando qualquer texto em um aplicativo e pressionando a tecla cadastrada!

---

## 🧪 Execução dos Testes Automatizados

O repositório possui cobertura completa de testes unitários para a interface gráfica, persistência de dados, telemetria, logs e injeção de atalhos.

Para executar toda a suíte de testes:

```bash
python -m unittest discover -s teste
```

Para executar um módulo de teste específico:

```bash
python -m unittest teste.test_gtray_ui
python -m unittest teste.test_gtray_data_manager
python -m unittest teste.test_system_prompt_integration
```

---

## 🔒 Segurança e Privacidade

- **Armazenamento Local:** Todas as chaves de API, credenciais e configurações são salvas exclusivamente de forma local no seu computador no arquivo `assets/gtray_data.json`.
- **Comunicação Direta:** As requisições de texto selecionado são enviadas diretamente da sua máquina para o endpoint do provedor de IA escolhido, sem intermediários.
- **Controle de Versão:** Arquivos contendo chaves locais e registros temporários estão devidamente protegidos no `.gitignore`.

---

## 📄 Licença

Distribuído sob a licença MIT. Consulte o arquivo `LICENSE` para mais informações.