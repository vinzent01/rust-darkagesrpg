# Rust Dark Ages RPG

Protótipo de jogo 2D em Rust com visual pixel art. O projeto explora movimentação em grade com animação, entidades que se movimentam por turnos, renderização de mapas e sprites, inventário e interação com objetos.

> **Estado atual:** protótipo em desenvolvimento. O código possui movimentação por turnos, combate corpo a corpo básico e coleta de recursos; crafting, salvamento e criação de personagem ainda não estão implementados.

## Índice

- [Requisitos](#requisitos)
- [Executar](#executar)
- [Controles](#controles)
- [Como funciona](#como-funciona)
- [Estrutura do projeto](#estrutura-do-projeto)
- [Recursos visuais e sonoros](#recursos-visuais-e-sonoros)
- [Dados e configuração](#dados-e-configuração)
- [Verificação](#verificação)
- [Limitações atuais](#limitações-atuais)

## Requisitos

- Rust estável com suporte à edição 2024 (Rust 1.85 ou mais recente).
- Cargo, incluído na instalação do Rust.

As dependências diretas estão declaradas em `Cargo.toml`:

- [`macroquad`](https://crates.io/crates/macroquad) — janela, loop de jogo, entrada, texturas e desenho 2D.
- [`hecs`](https://crates.io/crates/hecs) — ECS (Entity Component System) para entidades e sistemas do jogo.

## Executar

Na raiz do repositório:

```sh
cargo run
```

Para apenas compilar e verificar o código:

```sh
cargo check
```

Os arquivos de `content/` são carregados por caminhos relativos ao diretório de trabalho. Execute o programa a partir da raiz do repositório para que os caminhos dos recursos sejam encontrados.

## Controles

| Entrada | Ação |
| --- | --- |
| `W`, `A`, `S`, `D` | Move o personagem; cada passo toca o som de caminhada |
| Setas | Move o personagem |
| `H` | Alterna a visualização dos hitboxes dos sprites |
| Segurar `Shift` | Ativa o modo de ataque; clique em uma entidade para caminhar célula a célula e atacar corpo a corpo (20 de dano); vida atual/máxima aparece acima do jogador e das criaturas (100 máxima) |
| Segurar `Shift` e clicar em uma árvore ou toco com machado de pedra equipado | Golpeia a árvore com som; após quatro golpes, a copa tomba com som de queda, depois desvanece e concede madeira. Um golpe adicional remove o toco e concede mais 1 madeira |
| `I` | Abre ou fecha o inventário |
| Clique com o botão direito em uma roupa, ferramenta, arma ou escudo | Equipa ou desequipa o item; o inventário destaca os itens equipados |
| Clique esquerdo em um baú | Abre o baú e seus painéis de armazenamento |
| Clique e arraste no cabeçalho do inventário | Move a janela do inventário |
| Clique e arraste em um item do inventário | Move ou empilha o item; arrastá-lo para fora solta-o no chão |
| Roda do mouse sobre a lista do inventário | Rola os espaços do inventário |
| Clique e arraste entre os painéis do jogador e do baú | Transfere itens entre os inventários |
| Clique e arraste um item do baú para fora dos painéis | Solta o item no chão junto ao baú |
| Roda do mouse sobre a lista do baú | Rola os espaços de armazenamento |
| Clique em um item individual no chão, perto do personagem | Tenta recolher o item |
| Clique em `ItemStack` perto do personagem | Abre a lista dos itens naquela célula e o inventário |
| Arraste uma linha da lista para um slot compatível/vazio | Transfere aquele item para o inventário |
| Roda do mouse (com as interfaces fechadas) | Aproxima ou afasta a câmera |
| `M` | Abre ou fecha o painel de menu |

O painel de itens no chão pode ser fechado com `X` ou `Esc`. O menu atual é apenas um painel visual. Ele não pausa o mundo nem oferece opções adicionais.

## Como funciona

### Inicialização

1. Cria o mundo ECS e o banco de dados.
2. Carrega as texturas e registra as definições de tiles, raça, cabelo e itens.
3. Cria o jogador, um cavalo com sela, um baú, o sistema de geração de criaturas e o mapa.
4. Inicia o loop de atualização e desenho do Macroquad.

### Ciclo de jogo

Cada quadro atualiza o menu e as interfaces; o zoom da câmera é processado quando elas estão fechadas. Quando o jogador inicia um movimento válido, o jogo avança um turno: atualiza o contador de geração de criaturas e escolhe movimentos para as entidades com comportamento aleatório. O movimento é animado usando uma curva de Bézier e o turno termina quando não há entidades em movimento.

O movimento do jogador aceita as quatro direções cardeais e diagonais. O destino é limitado pelas dimensões do mapa e por entidades marcadas como sólidas ou como caminhantes aleatórios. Os caminhantes aleatórios escolhem entre células cardeais livres.

### Mapa e câmera

- O mapa inicial tem `100 × 100` células, com tamanho de `32 × 32` unidades de mundo.
- A câmera acompanha o jogador e o zoom é ajustável pela roda do mouse.
- O mapa é composto por camadas de tiles. A implementação inicial cria duas camadas preenchidas com os tiles de grama e objeto de grama.
- Árvores são distribuídas proceduralmente pelo mapa; cada uma combina a copa `pine-tree.png` com a base `cutted-tree.png`, bloqueia a passagem e deixa uma área livre ao redor do ponto inicial.
- A grade é desenhada como auxílio visual durante o jogo.

### Inventário e itens

- O inventário inicial tem 25 espaços exibidos como uma lista rolável; os espaços vazios também aparecem e podem receber itens.
- O inventário inicial inclui túnica, calças e botas equipáveis. Clique com o botão direito em cada peça para equipá-la ou desequipá-la; a linha fica destacada e a roupa aparece sobre o personagem. Equipar outra peça do mesmo tipo substitui visualmente a anterior.
- Picareta, machado de pedra e espada podem ser equipados na mão principal; o escudo ocupa a mão secundária. As armas e o escudo aparecem ao lado do personagem, trocando de lado com a orientação dele.
- O limite de peso do jogador é 20 kg. O cabeçalho mostra o peso carregado e cada item mostra o peso por unidade e o peso total da pilha.
- Árvores podem ser derrubadas com quatro golpes enquanto o machado de pedra estiver equipado; a madeira é adicionada ao inventário ou deixada no chão se não houver espaço.
- O baú possui um inventário independente com 25 espaços e sem limite de peso; seu painel mostra o peso armazenado.
- Com o baú aberto, é possível arrastar itens entre os painéis do jogador e do baú, ou soltar itens do baú no chão. Itens de uma pilha no chão também podem ser arrastados diretamente para o baú.
- Os itens possuem identificador, nome, peso e caminho para a textura.
- Itens iguais são empilhados ao recolher ou ao soltá-los em outro espaço.
- O peso máximo é verificado ao recolher ou transferir itens do chão.
- É possível arrastar itens para fora da janela para soltá-los em uma célula livre próxima ao jogador. Se a célula de origem estiver ocupada por um objeto, o item é colocado em outra célula livre. Cada unidade solta passa a ser um objeto individual, sem contador desenhado.
- Quando vários objetos estão na mesma célula, o chão mostra o sprite `content/ItemStack/ItemStack.png`. Clicar na pilha abre a lista com todos os itens nela; é possível rolar a lista e arrastar cada item para um espaço vazio ou para uma pilha do mesmo tipo no inventário.
- Um item individual no chão pode ser recolhido clicando na sua célula; pilhas e itens individuais só podem ser acessados a até duas células do jogador.

O conteúdo inicial do inventário é definido em `src/player.rs`; as definições e texturas dos itens são registradas em `src/database.rs`.

### Criaturas

As definições das criaturas (ID, nome, textura, vida máxima e peso de aparição) são registradas em `src/database.rs`, junto ao carregamento dos itens. O cavalo branco, o cavalo, o urso, o cervo, o grifo, o coelho e o lobo ficam disponíveis no campo `Database::creatures`. `spawner::spawn_creature` instancia qualquer criatura registrada em uma posição escolhida.

O spawner é atualizado a cada movimento válido iniciado pelo jogador. A cada dez turnos, tenta gerar uma criatura com base nos pesos relativos do registro, em uma célula livre do mapa e fora da área visível da câmera. O cavalo inicial e as criaturas geradas possuem componentes de movimento aleatório.

## Estrutura do projeto

```text
.
├── Cargo.toml          # Pacote, edição do Rust e dependências
├── Cargo.lock          # Versões resolvidas das dependências
├── Concept.ase         # Arquivo-fonte de arte
├── content/            # Imagens, shaders e efeitos sonoros
└── src/
    ├── main.rs         # Inicialização da aplicação e loop principal
    ├── body.rs         # Componentes e dados visuais de corpo, raça e aparência
    ├── database.rs     # Carregamento de recursos e registro de dados
    ├── game.rs         # Atualização por turnos, câmera e desenho das camadas do jogo
    ├── game/
    │   └── scene.rs    # Criação das entidades iniciais da cena
    ├── chest_inventory.rs # Interface e transferências do inventário do baú
    ├── hud.rs          # Interface, desenho e interação do inventário
    ├── input.rs        # Entrada de movimento, colisão e zoom da câmera
    ├── inventory.rs    # Itens, inventário, coleta e desenho de itens no chão
    ├── menu.rs         # Painel de menu alternado pela tecla M
    ├── movement.rs     # Movimento Bézier, caminhantes aleatórios e câmera
    ├── objects.rs      # Baú, sprite alternável e componente de solidez
    ├── player.rs       # Componentes e inventário inicial do jogador
    ├── render.rs       # Desenho de mapa, entidades, aparência e objetos
    ├── spawner.rs      # Contador por turnos e geração de criaturas
    ├── tilemap.rs      # Tipos e criação do mapa em camadas
    └── utils.rs        # Conversões de coordenadas, Bézier e grade
```

### Componentes e sistemas

O mundo usa `hecs`: componentes são estruturas de dados e os módulos consultam entidades que possuem os componentes necessários.

- `Position` armazena posição e deslocamento visual.
- `BezierMovement` guarda os pontos e o progresso da animação.
- `RandomWalker` marca entidades que recebem movimentos automáticos por turno.
- `Sprite`, `SpriteOverlay` e `Appearence` descrevem diferentes formas de desenhar entidades.
- `Solid`, `Chest` e `OnOffSprite` representam a colisão e a aparência alternável do baú.
- `Inventory`, `GroundItem`, `UIInventory` e `ChestInventoryUI` separam os dados dos itens no mundo e as interfaces do jogador, do chão e do baú.

`src/main.rs` coordena essas partes; cada módulo mantém as regras específicas do seu sistema.

## Recursos visuais e sonoros

`content/` contém 124 arquivos distribuídos por categorias, incluindo:

- `vegetation/` — tiles, vegetação, água, pedras, madeira e objetos do cenário.
- `creatures/` — criaturas e montarias.
- `chests/` — estados aberto e fechado do baú.
- `humans/`, `dwarves/`, `elves/` e `kalinkan/` — partes de personagens.
- `male clothes/`, `female clothes/`, `melee weapons/` e `shield/` — equipamentos e armas.
- `hud/` — slots e ícones da interface.
- `shaders/` — shaders de contorno e movimento de árvore.
- `sound-effects/` — efeitos sonoros.
- `crafting-items/`, `ActionsIcons/` e `ItemStack/` — recursos adicionais.

O banco de dados percorre recursivamente `content/` e carrega arquivos de imagem PNG, JPG, JPEG, BMP e TGA como texturas. Na configuração atual, o registro funcional de raça usa humano, e o cabelo carregado para a aparência do jogador é `medium-hair.png`; a presença de outros recursos na pasta não significa que já estejam conectados a um sistema do jogo. Os shaders e efeitos sonoros também estão presentes como recursos, mas ainda não são usados pelo fluxo principal atual.

## Dados e configuração

As definições de tiles e itens são registradas diretamente no código:

| Categoria | Identificadores atuais | Arquivo de registro |
| --- | --- | --- |
| Tiles | `1` grama, `2` objeto de grama, `3` terra | `src/database.rs` |
| Itens | `stick`, `stone_axe`, `pickaxe` | `src/database.rs` |
| Raça carregada | `human` | `src/database.rs` |
| Cabelo carregado | Cabelo humano usado pelo jogador | `src/database.rs` |

Para adicionar uma textura, coloque um arquivo de imagem em `content/`. Para que ela seja usada pelo jogo, também registre sua definição ou conecte-a ao sistema apropriado e use o caminho relativo esperado pelo código.

## Verificação

O repositório não contém atualmente uma suíte de testes automatizados. Para validar que o projeto compila:

```sh
cargo check
```

Para verificar a execução e os recursos visuais, inicie com `cargo run` a partir da raiz do repositório e teste os controles descritos acima.

## Limitações atuais

- O mapa e os dados iniciais são criados no código, sem carregamento de fases editáveis.
- Não há salvamento ou carregamento de progresso.
- O painel de menu não pausa a simulação.
- Não há combate, crafting, missões, diálogo ou progressão implementados.
- Várias categorias de recursos ainda não estão ligadas a sistemas funcionais.
