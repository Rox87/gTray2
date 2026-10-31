# Especificação de Design: Redesenho Visual do gTray

- **Data:** 12 de Outubro de 2026
- **Status:** Proposta para Aprovação
- **Autor:** Antigravity / Pair Programming

---

## 1. Contexto e Motivação (Intent & Why)

O **gTray** é um utilitário desktop para Windows que roda na bandeja do sistema e conecta a inteligência artificial (OpenAI) a qualquer aplicativo via atalhos de teclado.

Atualmente, a janela de configuração (`GTray.py`) apresenta um aspecto visual classificado pelo usuário como **"muito pesado"**:
- Títulos desproporcionais com fundo roxo e fontes de até 48px.
- Fundo azul royal (`#2B5DD1`) com bordas grossas de 6px.
- Botões em vermelho escuro (`darkred`), laranja (`#D2691E`) e verde cru.
- Um logotipo de 300x300 px ocupando a maior parte do espaço útil.
- Mecânica de desbloqueio da chave API pouco intuitiva (exige clicar na imagem para liberar a edição).

O usuário solicitou um visual **minimalista, moderno e profissional**, disponibilizando tanto o **Tema Escuro (Opção A)** quanto o **Tema Claro (Opção B)**, sendo o **Tema Escuro o padrão**.

---

## 2. Objetivos, Não-Objetivos e Anti-Objetivos

### Objetivos (Goals)
1. **Design System Moderno em PyQt5:** Implementar uma folha de estilos limpa (QSS) com tokens de cores consistentes, tipografia nativa (`Segoe UI` / `Inter`), cantos arredondados sutis (6-10px) e bordas finas (1px).
2. **Suporte a Dois Temas (Dark e Light):**
   - **Tema A (Dark Mode Studio - Padrão):** Fundo grafite/ardósia escuro (`#0F172A`), cartões em `#1E293B`, bordas `#334155`, textos claros `#F8FAFC`, acentos em azul `#3B82F6`.
   - **Tema B (Modern Clean Light):** Fundo cinza suave `#F8FAFC`, cartões em branco `#FFFFFF`, bordas sutis `#E2E8F0`, textos `#0F172A`, acentos em azul `#2563EB`.
3. **Alternador de Tema em Tempo Real:** Botão com ícones ☀️ / 🌙 no cabeçalho permitindo alternar instantaneamente entre escuro e claro sem fechar a aplicação.
4. **Persistência da Escolha do Usuário:** Salvar o tema selecionado em `assets/config.ini` (`tema = dark` ou `tema = light`), inicializando no tema salvo (ou dark por padrão).
5. **Reestruturação e Ergonomia dos Componentes:**
   - **Cabeçalho:** Ícone compacto e profissional do gTray (28x28 px), título limpo, badge de status ("Ativo" / "Pausado") e botão de alternância de tema.
   - **Chave de API:** Campo protegido (modo senha com caracteres ocultos), botão para visualizar (`👁`), botão para "Editar / Salvar" e link direto de ajuda/informações.
   - **Gerenciador de Atalhos:** Seletor de modo (Upgrade, Generic, Python, Creation), badge destacada com a combinação de teclas atual em fonte monoespaçada e botão de ação direta "Gravar Atalho".
   - **Rodapé de Ações:** Botão principal de controle do serviço (Iniciar / Pausar), checkboxes alinhados ("Iniciar com o Windows", "Manter Visível") e botão para minimizar para a bandeja.
6. **Dimensões e Proporções Compactas:** Janela com tamanho proporcional (~380x520 px), eliminando barras de rolagem desnecessárias e elementos esticados.

### Não-Objetivos (Non-Goals)
- Não alterar as bibliotecas subjacentes (manter PyQt5, pystray, keyboard).
- Não alterar as regras de negócio de chamada à IA em `handler.py`, `pessoal/IA.py` ou a captura de teclas em `services/klisten.py`.

### Anti-Objetivos (Anti-Goals)
- Não utilizar cores saturadas gritantes (roxo puro, azul royal, verde sem harmonia).
- Não exigir interação oculta (como clicar em imagens para desbloquear formulários).
- Não quebrar compatibilidade com a inicialização via linha de comando (`sys.argv[1] = "1"` ou `"2"`).

---

## 3. Especificação Técnica e de Layout

### 3.1 Arquitetura dos Estilos (QSS)
Para manter o código limpo e de fácil manutenção, criaremos um módulo dedicado `ui_theme.py` (ou dentro do próprio `GTray.py` em uma classe `ThemeManager` bem estruturada) que expõe:
- `get_stylesheet(theme_name: str) -> str`: Retorna a string QSS completa para `dark` ou `light`.
- Paleta semântica:
  - Fundo da Janela (`window-bg`)
  - Fundo do Card/Container (`card-bg`)
  - Bordas (`border-color`)
  - Textos principais e secundários (`text-primary`, `text-secondary`)
  - Cores de Acento (`accent-blue`, `hover-blue`, `accent-green`, `accent-red`)

### 3.2 Estrutura da Janela (`GTray_CFG_UI`)

```
+-------------------------------------------------------+
|  [G] gTray                [● Ativo]   [☀️/🌙 Tema]     |  <- Header
+-------------------------------------------------------+
|  CHAVE OPENAI                          [Editar] [?]   |
|  [ sk-••••••••••••••••••••••••• ] [👁]                 |  <- Card API Key
+-------------------------------------------------------+
|  ATALHOS GLOBAIS                                      |
|  Modo: [ Upgrade (Melhorar Texto)          ▼ ]        |  <- Card Atalhos
|  Combinação Atual:  [ Ctrl + Alt + U ]                |
|  [ Gravar Novo Atalho ]                               |
+-------------------------------------------------------+
|  [✓] Iniciar com o Windows    [✓] Manter Visível      |  <- Preferências
+-------------------------------------------------------+
|  [ Iniciar Serviço / Pausar ]    [ Minimizar ]        |  <- Ações Principais
+-------------------------------------------------------+
```

### 3.3 Tratamento da Chave de API
- Por padrão, a chave é lida de `assets/k.cfg`.
- Quando bloqueada: o campo `QLineEdit` fica em modo senha (`EchoMode.Password`), desabilitado para edição, exibindo bolinhas.
- Botão "Editar" alterna o campo para habilitado (`Enabled = True`) e modo texto visível (ou mantém opção de alternar visibilidade com o botão do olho). O botão passa a exibir "Salvar".
- Ao clicar em "Salvar", grava em `assets/k.cfg`, atualiza a instância `Gtray` e retorna ao modo de leitura seguro com confirmação visual suave.

### 3.4 Captura de Atalhos
- Mantém o fluxo com `KListen`, porém com feedback visual direto na própria caixa de atalhos (ex: "Pressione a combinação... (Esc cancela)").

---

## 4. O que Ficou a Cargo do Construtor (Builder's Calls)
1. **Módulo de Temas:** Criar `ui_theme.py` para isolar os estilos QSS, garantindo que `GTray.py` permaneça enxuto e legível.
2. **Ícones Vetoriais/SVG ou Emojis Modernos:** Utilizar símbolos de alta nitidez para os botões de visibilidade (`👁`), tema (`☀️`/`🌙`) e configurações.
3. **Persistência Segura no Config:** Se a chave `tema` não existir em `assets/config.ini`, assumir `dark` automaticamente e registrar a seção caso necessário.
4. **Transições de Botões:** Estilizar todos os pseudo-estados de botões (`:hover`, `:pressed`, `:disabled`) para fornecer feedback tátil consistente.

---

## 5. Plano de Validação
1. **Execução Local:** Iniciar o app com `python GTray.py` e verificar a renderização inicial no Tema Escuro (Padrão).
2. **Teste de Alternância:** Clicar no botão de tema e verificar a transição imediata para o Tema Claro e volta para o Tema Escuro.
3. **Persistência:** Fechar o app no tema claro, reabrir e verificar se iniciou no tema salvo.
4. **Funcionalidades:** Testar edição da chave de API, seleção de atalhos, gravação de atalho e botão de Iniciar/Pausar serviço.
