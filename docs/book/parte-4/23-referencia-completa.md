# Cap. 23 — Referência completa: sintaxe e recursos

> **Versão âncora:** Ori **0.3.8** — superfície **S3** + inferência local (opção B)
> **ABI:** `ori-native-abi-1` · **Backend semântico:** nativo (Cranelift AOT/JIT)
> **Parte:** IV — consulta
> **Verificação:** os exemplos completos deste capítulo foram compilados com
> `ori 0.3.8` (`ori check`); os que mostram resultado também foram executados com
> `ori run`. Trechos de assinatura e fragmentos ilustrativos aparecem marcados
> pelo contexto.

## TL;DR

Esta é a referência de consulta da linguagem: **toda a superfície sintática** e
**todos os recursos** implementados em 0.3.8, em uma ordem que segue o arquivo
`.orl` (cabeçalho → imports → tipos → declarações → corpo → recursos).

- Se você quer **aprender**, leia o [tour](../../language/tour.pt-BR.md) e a
  [Parte III](../parte-3/11-anatomia-programa.md).
- Se você quer **conferir uma forma exata**, use este capítulo.
- A fonte normativa continua sendo [`docs/spec/`](../../spec/README.md); este
  capítulo é a versão de leitura rápida, em português, verificada contra o
  compilador.

**O que memorizar primeiro:** blocos terminam em `end`; não existe `func` de
declaração; tipos compostos usam `[]`; ausência é `optional[T]`; falha é
`result[T, E]`; propagação é `try expr`; comportamento vem de
`apply Tipo: Trait`.

---

## Como ler esta referência

- `--` inicia comentário de linha em Ori. Comentários nos exemplos explicam a
  regra, não são saída de programa.
- Trechos marcados **ERRO** mostram código rejeitado; o código de diagnóstico
  aparece ao lado.
- Em código Ori, `...` é o marcador de parâmetro variádico. Em texto, `(...)`
  indica argumentos elididos, não sintaxe para copiar.
- Anotações como `T`, `E`, `K`, `V`, `N` são metavariáveis de documentação.
- Códigos como `parse.func_removed` podem ser consultados com
  `ori explain parse.func_removed`.

---

## Sumário

- [1. Estrutura de um arquivo](#1-estrutura-de-um-arquivo)
- [2. Imports e visibilidade](#2-imports-e-visibilidade)
- [3. Léxico](#3-léxico)
- [4. Tipos](#4-tipos)
- [5. Bindings e atribuição](#5-bindings-e-atribuição)
- [6. Funções](#6-funções)
- [7. Statements e controle de fluxo](#7-statements-e-controle-de-fluxo)
- [8. Expressões](#8-expressões)
- [9. Structs e enums](#9-structs-e-enums)
- [10. Traits e `apply`](#10-traits-e-apply)
- [11. Genéricos](#11-genéricos)
- [12. Erros, ausência e panic](#12-erros-ausência-e-panic)
- [13. Memória e limpeza](#13-memória-e-limpeza)
- [14. Concorrência](#14-concorrência)
- [15. Atributos e compilação condicional](#15-atributos-e-compilação-condicional)
- [16. FFI e ABI nativa](#16-ffi-e-abi-nativa)
- [17. Biblioteca padrão](#17-biblioteca-padrão)
- [18. Projeto, build e ferramentas](#18-projeto-build-e-ferramentas)
- [19. Diagnósticos](#19-diagnósticos)
- [20. Estabilidade, compatibilidade e migração](#20-estabilidade-compatibilidade-e-migração)
- [Apêndice A — Gramática resumida (não normativa)](#apêndice-a--gramática-resumida-não-normativa)
- [Apêndice B — Notas de fidelidade (0.3.8)](#apêndice-b--notas-de-fidelidade-038)
- [O que memorizar](#o-que-memorizar)
- [Para continuar](#para-continuar)

---

## 1. Estrutura de um arquivo

Um arquivo `.orl` é UTF-8 e tem três regiões, nesta ordem:

```text
module <caminho>          -- obrigatório, primeira declaração
imports                   -- zero ou mais, antes de qualquer declaração
declarações               -- tipos, funções, constantes, traits, apply, extern
```

```ori
module app.inventory

import ori.io = io

public item_count() -> int
    return 42
end
```

| Regra | Detalhe | Diagnóstico |
|---|---|---|
| `module` é obrigatório | primeira construção do arquivo | `parse.module_missing` |
| `module` vem primeiro | nada antes dele | `parse.module_not_first` |
| imports antes de declarações | não intercala | `parse.import_after_declaration` |
| um módulo por arquivo | o caminho do `module` é a identidade | — |
| extensão canônica | `.orl` | — |
| BOM | aceito e ignorado só no início | `lex.unexpected_character` |

O caminho do módulo é `snake_case` por segmento (`app.kanban.board`). Em projeto,
`source.root_namespace` no `ori.proj` define o prefixo esperado.

### 1.1 Declarações de topo

```ori
module app.decls

alias UserResult = result[User, string]     -- nome transparente
newtype UserId = int                        -- tipo nominal
const MAX_USERS: int = 100                  -- constante de módulo
var counter: int = 0                        -- variável de módulo (compartilhada)
struct User                                 -- produto
    name: string
end
enum Role                                   -- soma
    Admin
    Member
end
trait Named                                 -- contrato de comportamento
    name(self) -> string
end
apply User: Named                           -- implementação de trait
    name(self) -> string => self.name
end
extern "C"                                  -- símbolos nativos
    public abs_int(x: int) -> int
end
```

Cada declaração de topo aceita atributos (`@test`, `@cfg`, …), cobertos na
[seção 15](#15-atributos-e-compilação-condicional).

---

## 2. Imports e visibilidade

### 2.1 As três formas de import

| Intenção | Forma | Efeito |
|---|---|---|
| Seletiva | `import ori.fs (read_text, write_text)` | só os nomes listados entram no escopo |
| Alias de módulo | `import ori.io as io` ou `import ori.io = io` | use `io.print(...)` |
| Módulo inteiro | `import ori.io` | **nenhum** alias; use `ori.io.print(...)` |

```ori
module app.api

import ori.fs (read_text, write_text)
import ori.string as str
import ori.math = math
import ori.io

main()
    const text: result[string, string] = read_text("notes.txt")
    const upper: string = str.to_upper("ori")
    const pi: float = math.pi
    ori.io.println("sem alias local")
end
```

- A direção é sempre **caminho primeiro, alias depois**: `import ori.io as io`.
  A forma invertida (`import io = ori.io`, herança do Auk9) foi removida.
- `as` e `=` são equivalentes em imports.
- `import path only (...)` foi removido (`parse.import_only_removed`).

### 2.2 Renomear itens dentro da lista

```ori
import ori.string (is_empty, truncate = cut, to_upper as upper)
```

`item = alias` e `item as alias` funcionam dentro da lista seletiva.

### 2.3 Bloco `imports`

Vários imports em um bloco, fechado por `end`:

```ori
imports
    ori.io = io
    ori.fs = fs, ori.path = path
    ori.string (is_empty)
end
```

Vírgulas só são necessárias quando há **mais de um import na mesma linha**.

### 2.4 Reexportação

```ori
module app.api

public import app.inventory = inventory
public import ori.io = io
```

`public import` reexporta; `import` simples não reexporta. Quem importa
`app.api` enxerga `api.inventory.item_count()`.

### 2.5 Visibilidade

- Declarações de topo são **privadas por padrão**.
- `public` torna a declaração visível para outros módulos.
- Acessar item privado de outro módulo é `name.private`.
- Parâmetros e tipos de retorno de função pública fazem parte do contrato:
  nomes de parâmetros nomeados também.

### 2.6 Imports de projeto e de dependência

```ori
module app.main

import app.helper (twice)        -- módulo do mesmo projeto
import demo.math (double)        -- pacote dependente (prefixo do pacote)
```

Imports de projeto são resolvidos a partir do `ori.proj`; imports de pacote
exigem o prefixo do pacote declarado em `[dependencies]`. Um nome nu nunca é
procurado em todas as dependências.

### 2.7 Regras da biblioteca padrão

- Módulos da stdlib começam com `ori.` e precisam de import explícito.
- Módulo desconhecido: `bind.stdlib_module_unknown`.
- Módulo planejado/bloqueado: `bind.stdlib_module_unavailable`.
- Caminhos aninhados antigos (`ori.fs.utils`) continuam aceitos como
  compatibilidade silenciosa; código novo usa o pai (`ori.fs`).
- Import não usado gera aviso `bind.unused_import`.

---

## 3. Léxico

### 3.1 Espaço em branco e comentários

Ori não usa indentação como sintaxe. A indentação recomendada é **4 espaços**
(`ori fmt` aplica).

```ori
-- comentário de linha: vai até o fim da linha

--|
Comentário de bloco.
Não aninha.
|--
```

| Forma | Regra |
|---|---|
| `--` | comentário de linha |
| `--\| … \|--` | bloco; não aninha; sem fechar é `lex.unclosed_block_comment` |
| `--\| … \|--` antes de declaração | vira **documentação** para `ori doc` |

Comentário de documentação reconhece as tags `@param nome descrição` e
`@returns descrição`; tags desconhecidas ficam como texto.

```ori
--|
Calcula a área de um retângulo.
@param width  Largura em pixels (positiva).
@param height Altura em pixels (positiva).
@returns A área calculada.
|--
public area(width: int, height: int) -> int
    return width * height
end
```

### 3.2 Identificadores

```text
identificador = letra { letra | dígito | "_" }
letra         = qualquer letra Unicode ou "_"
```

| Forma | Convenção |
|---|---|
| Tipos, traits, enums, variantes | `PascalCase` |
| Funções, variáveis, módulos | `snake_case` |
| Constantes de módulo | `SCREAMING_SNAKE_CASE` (opcional) |

Shadowing no **mesmo escopo** é erro de compilação; em escopo interno é
permitido.

### 3.3 Palavras reservadas

Reservadas e sem uso como identificador:

```text
module    namespace import    as        public    func      return
end       const     var       if        else      elif      while
for       in        repeat    break     continue  match     case
loop      struct    trait     apply     use       implement enum
where     is        alias     newtype   do        and       or
not       true      false     none      success   error     some
mut       self      attr      extern    any       using     check
with      then      tuple     lazy      handle    optional  result
list      map       set       range     void      bool      string
bytes     int       int8      int16     int32     int64
u8        u16       u32       u64
float     float32   float64
```

Reservadas **de propósito** para dar diagnóstico dedicado a formas removidas:

| Palavra | Forma antiga | Forma canônica | Diagnóstico |
|---|---|---|---|
| `namespace` | `namespace app.main` | `module app.main` | `parse.namespace_removed` |
| `func` (declaração) | `func f(...)` | `f(...)` | `parse.func_removed` |
| `implement` | `implement Trait for T` | `apply T: Trait` | `parse.implement_removed` |
| `do` | `do(u) => …` | `(u) => …` | `parse.do_removed` |
| `where` | `where T is Trait` | `for T: Trait` | `parse.removed_where_bound` |
| `success` / `error` | construtores antigos | `ok` / `err` | `parse.result_ctor_renamed` |

`func` sobrevive apenas como **tipo chamável**: `func(int) -> int`.
`attr` está reservada sem forma de superfície hoje, para não ser uma mudança
quebradiça no futuro.

Palavras **contextuais** (viram identificador comum fora da posição):

| Palavra | Posição que a ativa |
|---|---|
| `async` | antes de declaração de função |
| `await` | antes de expressão, dentro de `async` |
| `ok` / `err` | construtores e padrões |
| `try` | prefixo de expressão |
| `it` | contrato `if` de campo/parâmetro |
| `times` | depois de `repeat expr` |
| `c` / `host` | rótulo de ABI em `extern` |
| `for` | cláusula de bounds: `max for T: Trait (...)` |
| `use` | seção dentro de `apply` |
| `imports` | abre bloco de imports |
| `array` / `simd` / `slice` | posição de tipo |
| `iter` | antes de declaração de gerador |
| `suspend` | início de statement dentro de `iter` |

### 3.4 Literais

**Booleanos:** `true`, `false`.

**Inteiros** — padrão `int` (64 bits com sinal):

```ori
0
42
1_000_000        -- separador
0xFF             -- hexadecimal
0b1010_1010      -- binário
0o755            -- octal
42i8  42i16  42i32  42i64
42u8  42u16  42u32  42u64
```

**Floats** — padrão `float` (64 bits):

```ori
3.14
1.0e10
1.0e-5
6.022_140_76e23
3.14f32
3.14f64
```

**Strings** — aspas duplas, com escapes:

| Escape | Significado |
|---|---|
| `\\` | barra invertida |
| `\"` | aspas |
| `\n` `\r` `\t` | nova linha, retorno, tabulação |
| `\0` | byte nulo |
| `\u{1F600}` | valor escalar Unicode |

```ori
"line one\nline two"
"unicode \u{1F600}"
```

**Strings multilinha** (aspas triplas):

```ori
const sql: string = """
    SELECT *
    FROM users
    """
```

- A abertura `"""` exige nova linha depois.
- O fechamento `"""` define a coluna-base: o espaço inicial até essa coluna é
  removido de todas as linhas.
- A linha do fechamento não entra no conteúdo.
- Escapes continuam valendo.

**Strings interpoladas** — prefixo `f`:

```ori
const name: string = "Ada"
const greeting: string = f"hello {name}"
const detail: string = f"score: {1 + 41}"
const multi: string = f"""
hello {name}
"""
```

Regras:

- `{expr}` interpola qualquer tipo que implemente `Displayable`.
- A expressão interpolada **não pode conter literal de string** (aspas
  internas quebram o literal): `f"{f'{1}'}"` é erro de léxico.
- `{{` e `}}` produzem chaves literais: `f"literal {{x}}"` vira `literal {x}`.

**Bytes** — prefixo `b`:

```ori
const raw: bytes = b"hello"
const hex: bytes = b"\xFF\x00"
```

`b"..."` não aceita `\u{...}` (`parse.byte_unicode_escape`); aceita escapes de
byte.

**Ranges** — inclusivos nas duas pontas, apenas `int`:

```ori
0..9     -- 0,1,…,9
5..3     -- 5,4,3 (decrescente)
7..7     -- um elemento
```

### 3.5 Operadores e pontuação

```text
.    ..    ->    =>    |>
+    -     *     /     %
+=   -=    *=    /=
==   !=    <     <=    >     >=
<<   >>
&    |     ^     ~
=
( )  [ ]  { }  :  ,  @
--|  |--
```

- `?` pós-fixo **não existe** (`parse.question_propagate_removed`); use `try`.
- `..` é range e fatia.
- `->` separa parâmetros do retorno.
- `=>` separa parâmetros do corpo de closure/função de expressão única.
- `|>` é o pipe.
- `@` prefixa atributos.

### 3.6 Precedência (da maior para a menor)

| Nível | Operadores | Associatividade |
|---|---|---|
| 1 | `.campo` `chamada()` `[índice]` | esquerda |
| 2 | `-` unário, `not`, `~`, `try`, `await` | prefixo |
| 3 | `*` `/` `%` | esquerda |
| 4 | `+` `-` | esquerda |
| 5 | `==` `!=` `<` `<=` `>` `>=` | **não encadeia** |
| 6 | `<<` `>>` | esquerda |
| 7 | `&` | esquerda |
| 8 | `^` | esquerda |
| 9 | `\|` | esquerda |
| 10 | `and` | esquerda |
| 11 | `or` | esquerda |
| 12 | `\|>` | esquerda |

Duas consequências práticas:

1. **Comparação não encadeia.** `a < b < c` é erro
   (`parse.chained_comparison`); escreva `a < b and b < c`.
2. **Comparação liga mais forte que bitwise** (diferente de C). `a & b == c`
   lê como `a & (b == c)` e normalmente falha com
   `type.bitwise_type_mismatch`. Escreva `(a & b) == c`.

```ori
1 + 2 * 3        -- 7
1 << 1 + 1       -- 4  (deslocamento depois da soma)
1 | 2 & 3        -- 3  (& antes de |)
```

---

## 4. Tipos

Ori é estaticamente tipada e **não** tem inferência global (Hindley–Milner).
Dentro de função, `const`/`var` podem omitir o tipo quando o lado direito é
localmente óbvio (seção 5.1).

### 4.1 Primitivos

| Tipo | Descrição | Tamanho |
|---|---|---|
| `bool` | `true`/`false` | 1 byte |
| `int` | inteiro com sinal de 64 bits (padrão) | 8 bytes |
| `int8` `int16` `int32` `int64` | inteiros com sinal | 1/2/4/8 |
| `u8` `u16` `u32` `u64` | inteiros sem sinal | 1/2/4/8 |
| `float` | IEEE 754 64 bits (padrão) | 8 bytes |
| `float32` `float64` | IEEE 754 | 4/8 |
| `string` | texto UTF-8 imutável, gerenciado | — |
| `bytes` | dados binários imutáveis, gerenciados | — |
| `void` | ausência de valor útil (só retorno) | 0 byte |

Todos copiam na atribuição. `int64` é alias de `int`; `float64` é alias de
`float`.

### 4.2 Struct, enum e tuple

```ori
struct Point
    x: int
    y: int
end

enum Shape
    Circle(radius: float)
    Rectangle(width: float, height: float)
    Point
end

const pair: tuple[int, string] = (1, "one")
```

- Struct é produto nomeado; todas as campos têm nome.
- Enum é soma; variantes unitárias ou com **campos nomeados**. Payload
  posicional é proibido (`Circle(float)` não existe).
- `tuple[A, B]` é produto posicional; acesso por `.0`, `.1`.

### 4.3 Coleções e tipos embutidos

| Tipo | Papel |
|---|---|
| `list[T]` | sequência redimensionável |
| `array[T, N]` / `array[T, size: N]` | tamanho no tipo, elementos **inline** |
| `buffer[T]` | bloco contíguo mutável de tamanho fixo, no heap |
| `slice[T]` | janela **somente leitura** O(1) sobre uma `list[T]` |
| `map[K, V]` | chave-valor |
| `set[T]` | valores únicos |
| `range[int]` | intervalo inclusivo |
| `optional[T]` | presença ou `none` |
| `result[T, E]` | `ok(T)` ou `err(E)` |
| `lazy[T]` | cálculo adiado, no máximo uma vez |
| `future[T]` | resultado assíncrono |
| `handle[T]` | identificador opaco emprestado de FFI |
| `any[Trait]` / `Trait` | despacho dinâmico por trait |
| `func(A, B) -> R` | valor chamável |
| `simd[T, N]` | vetor SIMD de largura fixa |
| `tuple[…]` | produto posicional |

`list`, `map`, `set`, `optional`, `result` e `range` são built-in: não exigem
import.

### 4.4 `optional[T]` e `result[T, E]`

```ori
const name: optional[string] = some("Ada")
const empty: optional[string] = none

const good: result[int, string] = ok(42)
const bad: result[int, string] = err("falhou")
```

Construtores: `some(v)`, `none`, `ok(v)`, `err(e)`. Em `result[void, E]`,
`ok()` sem argumento é válido.

Operações:

```ori
maybe.or("fallback")             -- valor ou fallback
find_user(id).or_return()        -- valor ou propaga none
loaded.or_wrap("contexto")       -- result[T,string]: adiciona contexto ao err
try expr                         -- desembrulha ou propaga
```

`or_return()` também existe para `result`; a forma antiga
`or_return(valor)` não é implementada. `or_wrap` só vale para erro `string`.

**Nota 0.3.8:** `.or_return()` é aceito sobre o resultado de uma **chamada**
(`find_user(id).or_return()`). Aplicado a um binding local
(`const o: optional[int] = …; o.or_return()`) o compilador reporta
`type.field_on_non_struct`; nesse caso use `try o`.

### 4.5 `range[int]`

```ori
const r: range[int] = 0..9

for i in r
    io.print(string(i))
end
```

- Não há `length()` nem `contains()` em v1.
- Em 0.3.8 os extremos **não** são acessíveis como campo: `r.start` e `r.end`
  falham com `type.field_on_non_struct` (e `end` é palavra reservada). Guarde
  os extremos em variáveis se precisar deles.
- Ranges de `float` não existem. Use `for` para iterar.

### 4.6 `lazy[T]`

```ori
const expensive: lazy[int] = lazy.once(() => compute())
const value: int = lazy.force(expensive)
```

O thunk roda no máximo uma vez; execuções seguintes devolvem o valor em cache.

### 4.7 `handle[T]`

Valor emprestado, em forma de ponteiro, para FFI. Não é dono, não retém `T`,
não é `Transferable`, e não há desreferência segura na linguagem.

```ori
import ori.handle = handles

const missing: handle[int] = handles.null()
const is_missing: bool = handles.is_null(missing)
```

`==`/`!=` comparam identidade do ponteiro, nunca desreferenciam.

### 4.8 `array[T, N]`: tamanho no tipo

```ori
struct Grid
    cells: array[int, 4]
    label: string
end

main()
    var xs: array[int, 3] = [1, 2, 3]
    xs[1] = 99
    const g: Grid = Grid { cells: [10, 20, 30, 40], label: "grid" }
    const third: int = g.cells[2]
end
```

Regras:

- `array[int, 4]` e `array[int, size: 4]` são o **mesmo** tipo.
- `array[int, 4]` e `array[int, 8]` são tipos diferentes.
- O literal precisa ter o tamanho exato (`type.array_length_mismatch`).
- Índice constante é checado em compilação (`type.array_index_out_of_bounds`).
- O elemento precisa ser **inline**: primitivos, `array` de inline e struct
  cujos campos sejam todos inline. Struct com `string`, `list` ou campo
  gerenciado é rejeitada (`type.array_element_not_inline`). Struct recursiva
  também.
- `mem.size_of` reporta o bloco inteiro (ex.: 32 para `array[int, size: 4]`).

### 4.9 `simd[T, N]`: vetores portáveis

```ori
const a: simd[float32, 4] = [1.0f32, 2.0f32, 3.0f32, 4.0f32]
const b: simd[float32, 4] = [10.0f32, 20.0f32, 30.0f32, 40.0f32]
const c: simd[float32, 4] = a + b
const lane: float32 = c[0]
```

| Elemento | Lanes |
|---|---|
| `float32`, `int32` | 2, 4, 8, 16 |
| `float64`, `int64` | 2, 4 |
| `int16`, `u16` | 4, 8, 16 |
| `int8`, `u8` | 8, 16 |

Aceita `simd[T, N]` ou `simd[T, lanes: N]`. Operadores `+ - * /` operam lane a
lane; `c[i]` extrai uma lane.

**Atenção:** `u32` e `u64` **não** são aceitos como elemento de `simd` em 0.3.8
(`type.invalid_simd_type`); use `int32`/`int64` e converta na leitura.

### 4.10 `buffer[T]`: bloco contíguo mutável

```ori
import ori.buffer as buf

main()
    var pixels: buffer[u32] = buf.new(1920 * 1080)
    pixels[0] = 0xFF00FF00u32
    const v: u32 = pixels[0]
    const n: int = buf.len(pixels)
    buf.fill(pixels, 0u32)
end
```

- Elementos seguem a mesma regra inline de `array`.
- Tamanho fixado na alocação; não cresce.
- Indexação e atribuição por índice com checagem de limites.
- `buf.as_slice(b)` devolve `slice[T]`.

### 4.11 `slice[T]`: janela O(1)

```ori
import ori.list = lists
import ori.slice = sl

var xs: list[int] = [10, 20, 30, 40, 50]
const w: slice[int] = lists.window(xs, 1, 4)
const n: int = sl.len(w)          -- 3
const first: int = sl.get(w, 0)   -- 20
```

| Chamada | Resultado | Custo |
|---|---|---|
| `lists.slice(xs, a, b)` | nova `list[T]`, copiada | O(n) |
| `lists.window(xs, a, b)` | `slice[T]` sobre `xs` | O(1) |

A janela é **somente leitura** e observa escritas posteriores na lista dona. Ela
registra uma aresta ARC para a lista, então pode sobreviver ao binding que a
criou. A lista pode crescer por baixo; leituras além do tamanho atual abortam com
mensagem de limites. `slice` não é `Transferable`.

### 4.12 `any[Trait]` e nome de trait nu

```ori
const shape: Drawable = Circle { radius: 10.0 }
shape.draw()

draw_item(item: Drawable)
    item.draw()
end
```

- `any[Trait]` e `Trait` na posição de tipo são **o mesmo** tipo dinâmico.
- Prefira genéricos (`for T: Trait`) em caminho quente.
- `==` compara pelo vtable: payloads concretos iguais comparam estruturalmente;
  tipos concretos diferentes comparam diferentes.
- Passar `any[Trait]` por FFI exige anotação explícita.

### 4.13 Tipo chamável (`func`)

```ori
const double: func(int) -> int = (x: int) => x * 2
const notify: func(string) = (s: string) => io.print(s)   -- retorno void implícito
var handler: func(string) -> bool = (s: string) => len(s) > 0
```

### 4.14 `alias` e `newtype`

```ori
alias UserMap  = map[int, string]   -- outro nome para o mesmo tipo
newtype UserId = int                -- tipo distinto, mesma representação
```

| Forma | Semântica |
|---|---|
| `alias X = T` | transparente: `X` e `T` são intercambiáveis |
| `newtype X = T` | nominal: `UserId` não aceita `int` nem outro newtype |

```ori
const id: UserId = UserId(7)   -- converte para dentro
const raw: int = int(id)       -- converte para fora
```

`newtype` é custo zero (apagado no HIR). Não aceita parâmetros de tipo.

### 4.15 Igualdade (`==`)

| Tipo | Comportamento |
|---|---|
| numéricos, `bool`, `string`, `bytes` | igualdade de valor/conteúdo |
| `list[T]` | estrutural, ordem importa |
| `map[K,V]` | estrutural, ordem de inserção não importa |
| `set[T]` | estrutural, ordem não importa |
| `optional[T]`, `result[T,E]` | estrutural |
| `tuple[...]`, structs | campos em ordem de declaração |
| `any[Trait]` | via vtable |
| `func(...)` | erro de compilação |

Struct com campo incomparável (`func`, trait object sem igualdade) não aceita
`==`. Para igualdade própria, implemente `core.Equatable`:

```ori
import ori.core = core

apply User: core.Equatable
    equals(self, other: User) -> bool
        return self.id == other.id
    end
end
```

### 4.16 Conversões

Não há coerção implícita. Toda conversão é escrita:

```ori
const n: int = 42
const back: int = int(7u64)    -- valor sem sinal -> int
const s: string = string(42)   -- escalar → texto
const f: float = float(n)      -- inteiro → float
const small: u8 = 200u8        -- literal estreito usa sufixo
```

Limites reais em 0.3.8:

- `int(v)` converte um valor **inteiro** (qualquer largura/sinal) para `int`.
- `float(v)` converte **inteiro → float**; `float` de `float32` não existe.
- Não existe `int(3.9)` (float → int) nem `u8(v)`/`i8(v)`/`u16(v)`/`u32(v)`.
  Para valores estreitos, use literal com sufixo (`42u8`), `bytes.get` ou
  operações bit a bit.
- **Literal inteiro não adapta** para `u8`/`u16`: `const a: u8 = 42` é
  `type.type_mismatch`; escreva `42u8`. Fora do intervalo é
  `type.numeric_literal_out_of_range`.

`string(v)` aceita escalares e tipos com `Displayable`. Struct sem
`Displayable` não converte (`type.arg_type_mismatch`).

### 4.17 `is`: teste de tipo

```ori
if shape is Circle
    -- shape continua sendo any[Trait]; is NÃO estreita o tipo
end
```

- Produz `bool`.
- À direita, tipo nomeado (struct ou primitivo).
- Com `any[Trait]`, responde em runtime; com valor concreto, em compilação.
- **Não** testa variante de enum: use `match`.

### 4.18 Compatibilidade de tipos

1. `result[T, E]` só é compatível com `result[T, F]` se `E == F`.
2. `list[T]` só é compatível com `list[U]` se `T == U` (sem covariância).
3. Parâmetros genéricos são invariantes.
4. `any[Trait]` aceita qualquer tipo que implemente a trait.
5. `func(T) -> R` exige assinatura idêntica.

---

## 5. Bindings e atribuição

### 5.1 `const` e `var`

```ori
main()
    const name: string = "Ada"     -- imutável
    var count: int = 0             -- mutável
    count = count + 1
    count += 1
    count -= 1
    count *= 2
    count /= 2
end
```

- `const` não pode ser reatribuído (erro de compilação).
- `var` aceita `=`, `+=`, `-=`, `*=`, `/=`.
- O inicializador roda uma vez, na declaração.
- No topo do módulo, `public const` e `public var` são permitidos.
- `var` de módulo é compartilhado por todas as threads e **não** tem
  sincronização implícita.

### 5.2 Inferência local (opção B)

Dentro de função, o tipo pode ser omitido quando o lado direito é localmente
óbvio:

```ori
main()
    const n = 1
    const label = "ready"
    const user = User { name: "Ada", age: 36 }
    const xs = [1, 2, 3]
    const title = user.name            -- campo
    const first = xs[0]                -- índice
    const doubled = double(21)         -- chamada com retorno conhecido
    const piped = 21 |> double         -- pipe
    const maybe = some(10)             -- chamada
end
```

O tipo continua **obrigatório** quando:

- a declaração é `public` / de topo;
- o lado direito é `try expr`, `[]`/`{}` vazios, `none` nu, resultado `void`
  ou tipo não concreto;
- o tipo só seria descoberto por uso **posterior** no bloco.

```ori
-- ERRO: type.local_inference_failed
const e = err("x")
```

Nesse caso escreva `const e: result[int, string] = err("x")`.

### 5.3 Desestruturação de struct

```ori
struct Point
    x: int
    y: int
end

main()
    const Point { x, y } = get_pos()      -- com tipo
    const { x, y } = get_pos()            -- tipo inferido
    const Point { x: px, y: py } = pos    -- renomeando
    var { x, y } = get_pos()              -- bindings mutáveis
end
```

- Só campos de struct. Tuplas **não** são desestruturadas.
- Campo inexistente: `type.unknown_field`.
- Alvo que não é struct: `type.destructure_not_struct`.

### 5.4 Escopo e shadowing

Blocos aninham escopos (`if`, `while`, `for`, `loop`, `match`, corpo de função,
closure). Shadowing em escopo interno é permitido; no mesmo escopo é erro.

---

## 6. Funções

### 6.1 Declaração

Não existe palavra `func` na declaração. A forma é nome, parâmetros, retorno e
corpo:

```ori
add(a: int, b: int) -> int
    return a + b
end

public greet(name: string) -> string
    return f"hello {name}"
end
```

- `public` exporta para outros módulos.
- Sem `->`, o retorno é `void`.
- O corpo multilinha fecha com `end` (rótulo opcional: `end`; ver 6.11).
- `func nome(...)` é erro `parse.func_removed`.

### 6.2 Corpo de expressão única (`=>`)

```ori
double(x: int) -> int => x * 2

greet(name: string) -> string => f"hi, {name}"
```

### 6.3 Parâmetros

**Obrigatórios:**

```ori
connect(host: string, port: int) -> result[void, string]
```

**Com valor padrão** (depois dos obrigatórios):

```ori
connect(host: string, port: int = 80) -> result[void, string]
```

O padrão é avaliado **em cada chamada**, não uma vez.

**Nomeados** na chamada:

```ori
connect(host: "localhost", port: 8080)
connect(port: 8080, host: "localhost")   -- ordem livre
```

Depois do primeiro argumento nomeado, os seguintes também precisam ser
nomeados. Nomes de parâmetros de função pública fazem parte da API.

**Variádico** (só o último parâmetro):

```ori
log(prefix: string, values: string...)

const parts: list[string] = ["a", "b", "c"]
log("x", ..parts)          -- espalha uma lista
```

Dentro do corpo, `values` é `list[string]`. O espalhamento usa `..`.

**Contrato de valor** (`if it …`):

```ori
sqrt(value: float if it >= 0.0) -> float
clamp(v: int, lo: int, hi: int if it >= lo) -> int
```

`it` é o valor do parâmetro. Violação é panic de runtime
(`contract.param_violation`).

### 6.4 Retorno

```ori
area(w: int, h: int) -> int
    return w * h
end

log_all(items: list[string])
    for item in items
        io.print(item)
    end
    return              -- void explícito
end
```

- Função não-`void` que termina sem `return` é erro (`type.missing_return`).
- Em `result[void, E]`, o sucesso é `return ok()`.
- A forma de propagação é `try expr` (seção 12).

### 6.5 Métodos e `mut`

Métodos inerentes ficam **dentro** do `struct`:

```ori
struct Counter
    value: int

    get(self) -> int
        return self.value
    end

    mut increment(self)
        self.value = self.value + 1
    end

    make() -> Counter               -- função associada: sem self
        return Counter { value: 0 }
    end
end
```

- `mut` permite atribuir a `self` e seus campos.
- Método sem `mut` não pode modificar `self`.
- Chamar método `mut` em binding `const` é erro.
- Método sem `self` é **função associada**: chame `Counter.make()`.
- `self` implícito não existe: usar `self` sem declarar é erro.

### 6.6 Closures

```ori
const double: func(int) -> int = (x: int) => x * 2
const add: func(int, int) -> int = (a, b) => a + b

const blocky: func(int) -> int = (x: int)
    const y: int = x + 1
    return y
end
```

Regras:

- `(params) => expr` é corpo de expressão.
- `(params) … end` é corpo em bloco; aceita `-> T` explícito.
- Tipos de parâmetro podem ser omitidos **quando o contexto fornece** um
  `func(...)` de mesma aridade (binding anotado, parâmetro de `iter.map`, …).
- Sem contexto, `const f = (x) => x` falha com `type.local_inference_failed`.
- Captura é **por valor**.
- `do(...) =>` é erro `parse.do_removed`.

### 6.7 Chamadas poéticas

Uma chamada pode dispensar parênteses quando há **um** argumento na mesma
linha:

```ori
print_name "Ada"
io.println("Ada")        -- forma normal, sempre válida
```

- No máximo **uma** justaposição por expressão.
- Aninhar é erro `parse.poetic_call_nested`: `print greet name` não compila.

### 6.8 Genéricos em funções

Duas formas, **mutuamente exclusivas**:

```ori
identity[T](value: T) -> T
    return value
end

max for T: core.Comparable (a: T, b: T) -> T
    if a.compare(b) >= 0
        return a
    end
    return b
end
```

| Forma | Declara | Bounds |
|---|---|---|
| `nome[T, U](...)` | lista de tipos | nenhum |
| `nome for T: Trait (...)` | cláusula de bounds | sim |

- Misturar as duas é erro de parse.
- Com cláusula `for`, **todo** parâmetro de tipo precisa aparecer nela.
- A trait do bound precisa estar **no escopo** (`core.Comparable` ou importada
  por nome).

Detalhes em [seção 11](#11-genéricos).

### 6.9 Funções assíncronas

```ori
import ori.task = task

async load_count() -> int
    await task.sleep(1)
    return 41
end

async main()
    const count: int = await load_count()
end
```

- Chamar `async f(...) -> T` devolve `future[T]` imediatamente.
- `return valor` dentro do corpo ainda produz `T`.
- `await expr` só existe dentro de `async`, e `expr` precisa ser `future[T]`.
- `async main()` é permitido no backend nativo.
- `using` funciona dentro de `async`.
- Formas fora do subconjunto suportado falham com `backend.native_unsupported`
  antes de gerar código — nunca mudam a semântica em silêncio.

### 6.10 Geradores (`iter` + `suspend`)

```ori
iter counter(stop: int) -> int
    var i: int = 0
    while i < stop
        suspend i
        i = i + 1
    end
end

main()
    for n in counter(4)
        io.println(string(n))     -- 0, 1, 2, 3
    end
end
```

- O tipo declarado é o **elemento**, não a coleção.
- `suspend expr` entrega um valor ao `for` e retoma no próximo passo.
- `return` sem valor encerra a sequência; `return valor` é erro
  (`type.iter_return_value`).
- O iterador **não é valor**: só pode ser consumido direto no cabeçalho do
  `for` (`type.iter_call_outside_for`).
- A implementação é por inlining: sem alocação, sem máquina de estados.
  `break`, `continue` e `return` do corpo consumidor funcionam como em um `for`
  comum.
- Contratos de parâmetro continuam valendo (o compilador sintetiza o `check`).

Limites atuais, cada um com diagnóstico próprio:

| Limite | Diagnóstico |
|---|---|
| só função livre, sem método | `type.iter_method_unsupported` |
| mesmo módulo do `for` consumidor | `type.iter_cross_module_unsupported` |
| sem genéricos | `type.iter_generic_unsupported` |
| sem variádico | `type.iter_variadic_unsupported` |
| sem autorrecursão | `type.iter_recursive_unsupported` |
| não pode ser `async` | `parse.async_iter_unsupported` |

Quando o produtor precisa ser um **valor** (guardado, passado adiante,
consumido em pontos arbitrários), implemente `core.Iterable` com um struct de
estado explícito (seção 7.5).

### 6.11 `end` rotulado

```ori
if count > 0
    process()
end if

match shape
    case Circle(radius):
        draw_circle(radius)
    case else:
        draw_point()
end match

for item in items
    process(item)
end for
```

O rótulo é opcional. Rótulo que não corresponde à construção de abertura é
`parse.end_label_mismatch`.

### 6.12 Ponto de entrada

```ori
main()
    -- síncrono, sem retorno
end

main() -> result[void, string]
    return ok()
end

async main()
    await task.sleep(1)
end
```

O backend nativo aceita `main` síncrono, `main` que retorna
`result[void, E]` e `async main()`.

---

## 7. Statements e controle de fluxo

### 7.1 `if` / `elif` / `else`

```ori
if score >= 90
    io.println("A")
elif score >= 80
    io.println("B")
else
    io.println("C")
end
```

- A condição precisa ser `bool`; não há truthiness.
- `else if` foi removido (`parse.else_if_removed`); use `elif`.
- `if` de statement não produz valor; para valor use a forma inline
  `if cond then a else b` (seção 8).

### 7.2 `if some` / `if ok` / `if err`

```ori
if some(user) = find_user(id)
    greet(user)
else
    io.println("not found")
end

if ok(value) = divide(10, 2)
    io.println(string(value))
end

if err(message) = divide(1, 0)
    io.println(message)      -- entra quando NÃO é ok
end
```

- `if some(x) = e` liga o valor interno de `optional[T]`.
- `if ok(v) = e` liga `T`; `if err(e2) = e` liga `E` e entra no ramo de falha.
- O binding vive só no bloco do `if`.
- Scrutinee errado: `type.ifok_not_result`, `type.iferr_not_result`.

### 7.3 `while` e `while some`

```ori
while count < 10
    count += 1
end

while some(line) = reader.next_line()
    process(line)
end
```

`while some` repete enquanto a expressão der `some(v)`; para em `none`.

### 7.4 `loop`

```ori
loop
    const input: string = try console.read_line()
    if input == "quit"
        break
    end
    process(input)
end
```

Loop infinito; sai com `break`, `return` ou panic. `continue` pula para a
próxima volta.

### 7.5 `for`

```ori
for item in items
    process(item)
end

for item, index in items
    io.println(f"{index}: {item}")
end

for i in 0..9
    io.print(string(i))
end

for i in 9..0            -- decrescente
    io.print(string(i))
end
```

Iteráveis embutidos: `list[T]`, `map[K,V]`, `set[T]`, `string`, `bytes`,
`range[int]` e os handles opacos da stdlib (`deque`, `queue`, `stack`, listas
ligadas, `hash_table`, `graph`, `heap`).

Segundo binding:

- `list`, `set`, `string`, `bytes`, ranges, iteráveis customizados → índice
  `int` a partir de zero;
- `map[K,V]` → o **valor** `V` (o primeiro binding é a chave `K`).

```ori
const ages: map[string, int] = {"Ada": 36, "Bo": 41}

for name, age in ages
    io.println(f"{name} tem {age}")
end
```

**Iterável customizado (lazy):**

```ori
import ori.core = core

struct Counter
    current: int
    stop: int
end

apply Counter use core.Iterable
    mut next(self) -> optional[int]
        if self.current >= self.stop
            return none
        end
        const value: int = self.current
        self.current = self.current + 1
        return some(value)
    end
end

main()
    var c: Counter = Counter { current: 0, stop: 4 }
    for n in c
        io.println(string(n))
    end
end
```

`next` é chamado uma vez por passo: nada é materializado antes. `core.Iterable`
é reconhecida **estruturalmente** (pelo método `mut next() -> optional[T]`), não
pela tabela de traits. Sem `next`, o erro é `type.iterable_next_missing`.

### 7.6 `repeat`

```ori
repeat 3
    attempt()
end

repeat 3 times          -- `times` é opcional, só para leitura
    attempt()
end
```

A contagem é avaliada uma vez, precisa ser inteira; zero não itera; negativo é
panic de runtime.

### 7.7 `match`

```ori
match shape
    case Circle(radius):
        draw_circle(radius)
    case Rectangle(width, height):
        draw_rect(width, height)
    case Point:
        draw_point()
end
```

Regras:

- `case ... :` exige **dois-pontos** (omissão é erro de parse).
- Variantes de enum vão **sem** ponto inicial dentro do `match`
  (`case North:`); com ponto é `parse.case_dot_variant_removed`.
- `match` precisa ser exaustivo (`match.non_exhaustive`).
- `case else:` é o fallback explícito.
- Guardas: `case n if n >= 90:`; guarda falsa cai para o próximo caso.
- Casos com guarda **não contam** para exaustividade.
- Casos são testados em ordem de escrita; o primeiro que casa executa.
- Bindings do caso vivem no corpo e na guarda daquele caso.
- Caso inalcançável gera aviso.

**Padrões disponíveis:**

| Padrão | Forma |
|---|---|
| curinga | `_` |
| literal | `case 1:` / `case "a":` |
| variante unitária | `case North:` |
| variante com payload | `case Circle(radius):` ou `case Circle(radius: r):` |
| optional | `case some(v):` / `case none:` |
| result | `case ok(v):` / `case err(e):` |
| tupla | `case tuple(a, b):` |

Em payload nomeado, `case Circle(radius)` liga o campo `radius` a uma variável
de mesmo nome; `case Circle(radius: r)` renomeia para `r`.

**Alternativas (`or`):**

```ori
match direction
    case North or South:
        move_vertical()
    case East or West:
        move_horizontal()
end
```

- O separador é a palavra `or`; `case a, b:` é erro.
- Alternativas **não podem ligar valores**
  (`match.or_pattern_binding`).
- Para cobertura, `case a or b:` conta como `case a:` + `case b:`.

### 7.8 `using`

```ori
import ori.fs = fs
import ori.string = str

read_head(path: string) -> result[string, string]
    using file: fs.File = try fs.open_read(path)
    const raw: bytes = try fs.read(file, 256)
    return str.from_bytes(raw)
end
```

- O tipo é **obrigatório** na anotação (`using nome: Tipo = expr`).
- O `dispose()` roda em **toda** saída de escopo: `end` normal, `return`,
  propagação por `try`, `break`/`continue` e panic.
- Vários `using` são descartados em ordem inversa (LIFO):

```ori
main()
    using a: Res = open_a()
    using b: Res = open_b()
    -- na saída: b.dispose(), depois a.dispose()
end
```

- O tipo precisa implementar `core.Disposable`; senão `using.not_disposable`.
- Se a expressão falha antes do binding, nada é descartado.

### 7.9 `check`

```ori
check count >= 0
check index < ori.list.len(items), "index fora do intervalo esperado"
```

- Falha é **panic** não recuperável; não é tratamento de erro de negócio.
- A mensagem opcional precisa ser **literal de string**
  (`parse.check_message_literal`).
- Saída típica: `ori panic: check failed: index fora do intervalo esperado`.
- Use `result[T, E]` para falha esperada.

### 7.10 `break`, `continue`, `return`

```ori
loop
    if done
        break
    end
    if skip_this
        continue
    end
    process()
end
```

`break` e `continue` valem para o laço mais interno. `return` sai da função.

### 7.11 Expressão como statement

```ori
io.print("hello")
lists.push(items, item)
```

O valor é descartado. Se a expressão é `result[T, E]` e o resultado é
descartado sem tratamento, o compilador avisa (`type.unused_result`).

---

## 8. Expressões

### 8.1 Aritmética

```ori
a + b      -- Addable
a - b      -- Subtractable
a * b      -- Multiplicable
a / b      -- Divisible
a % b      -- resto
-a         -- negação
```

- Em primitivos, os operadores funcionam direto.
- Em tipos do usuário, exigem a trait correspondente.
- Divisão inteira trunca em direção a zero; divisor zero é panic de runtime.
- Divisão de `float` por zero segue IEEE 754 (`Infinity`/`NaN`).

### 8.2 Bitwise e deslocamento (0.3.8)

```ori
a & b       -- AND bit a bit: tipos inteiros iguais
a | b       -- OR
a ^ b       -- XOR
~a          -- complemento: x ^ -1 na largura do operando
a << n      -- deslocamento à esquerda
a >> n      -- à direita
```

- `&`, `|`, `^` exigem os **mesmos** tipos inteiros; o resultado tem esse tipo.
- `<<`/`>>` aceitam contagem de qualquer largura inteira; o resultado mantém o
  tipo da esquerda.
- `>>` é **aritmético** em tipo com sinal (preenche o bit de sinal) e
  **lógico** em tipo sem sinal (preenche zero).
- Contagem fora de `0..bit_width` é panic de runtime
  (`ori shift count out of range: must be in 0..bit_width`); em constante
  (CT-0) é erro de compilação.
- `~0u8 == 255`, `~0 == -1`.
- Bitwise **não** é sobrecarregável e não se mistura com `and`/`or`/`not`.

Exemplo típico, empacotar RGBA:

```ori
pack_rgba(r: u32, g: u32, b: u32, a: u32) -> u32
    return (r << 24) | (g << 16) | (b << 8) | a
end

const red: u32 = pack_rgba(255u32, 0u32, 0u32, 255u32)
```

### 8.3 Comparação e lógica

```ori
a == b   a != b   a < b   a <= b   a > b   a >= b
a and b  a or b   not a
```

- Todas produzem `bool`.
- `and`/`or` fazem curto-circuito.
- Comparação não encadeia.
- Tipo do usuário: `==`/`!=` via `core.Equatable`; ordenação via
  `core.Comparable`.

### 8.4 Acesso a campo

```ori
user.name
config.timeout
point.x
pair.0            -- tupla: índice inteiro literal
```

### 8.5 Índice e fatia

```ori
items[0]          -- T
items[2..5]       -- list[T] com 2, 3, 4 (fim exclusivo)
items[..5]        -- do início até 4
text[0..3]        -- caracteres 0, 1, 2 (escalares Unicode)
text[0]           -- string de um escalar
```

- Índice fora do intervalo é panic de runtime.
- Fatia de lista: `0 <= start <= end <= len`; violação é panic.
- `string` indexa por **valor escalar Unicode**, não por byte.
- `bytes` **não** suporta indexação (`type.not_indexable`); use `bytes.get`.
- **Nota 0.3.8:** as formas abertas `items[2..]` e `items[..]` passam na
  checagem, mas o runtime aborta com
  `ori list slice bounds out of range`. Use `[2..len]` com o fim explícito
  (`ori.list.len(items)`) até a correção.

### 8.6 Chamadas

```ori
add(1, 2)
io.print("hello")
user.display()
connect(host: "localhost", port: 8080)
concat(..parts)
```

Depois do primeiro argumento nomeado, todos os seguintes precisam ser nomeados.
`..lista` espalha em parâmetro variádico.

### 8.7 `await`

```ori
const value: int = await compute()
await task.sleep(1)
```

`await expr` espera `future[T]` e produz `T`. Só dentro de função `async`;
esperar algo que não é `future[T]` é erro de compilação.

### 8.8 `try` — propagação

```ori
const value: T = try fallible()     -- result[T, E]
const present: T = try maybe        -- optional[T]
```

| Tipo da expressão | Em sucesso | Em falha |
|---|---|---|
| `optional[T]` | desembrulha `T` | `return none` |
| `result[T, E]` | desembrulha `T` | `return err(e)` |

O retorno da função que contém precisa ser compatível: `optional[_]` para
`optional`, `result[_, E]` para `result`. `try` em função `void` é erro
(`type.propagate_return_mismatch`).

### 8.9 Pipe (`|>`)

```ori
const doubled: int = 21 |> double          -- double(21)
const upper: string = "ori" |> str.to_upper
```

- `a |> f` equivale a `f(a)`; o lado direito é uma **função** (nome ou caminho
  de módulo), não uma chamada.
- `a |> f(b)` **não** funciona em 0.3.8: o checker avalia `f(b)` como chamada
  normal e reporta `type.arg_count_mismatch`/`type.type_mismatch`. Chame a
  função diretamente quando ela recebe mais de um argumento.
- É tipado como chamada normal, então a inferência local vale quando o retorno é
  concreto.
- É o operador de menor precedência; na dúvida, use parênteses.

### 8.10 `if` inline

```ori
const label: string = if active then "on" else "off"
```

- Os dois ramos produzem o mesmo tipo.
- `else` é obrigatório (`parse.missing_else_in_if_expr`).
- A condição é `bool`.
- Em posição de statement, use o `if`/`elif`/`end` da seção 7.1.

### 8.11 `match` como expressão

```ori
const label: string = match n
    case 1: "one"
    case n if n >= 10: "big"
    case else: "other"
end
```

- Cada braço é **uma expressão**; todos produzem o mesmo tipo
  (`type.match_arm_mismatch`).
- Vale a mesma exaustividade do `match` de statement; guardas não contam, então
  `case else:` costuma ser necessário.
- Braço divergente (`return`) não restringe o tipo do resultado.

### 8.12 Literal de struct

Forma completa, sempre válida:

```ori
const p: Point = Point { x: 0, y: 0 }
const u: User  = User { name: "Ada", age: 36 }
```

Forma anônima, quando o tipo vem do contexto:

```ori
const p: Point = { x: 0, y: 0 }

origin() -> Point
    return { x: 0, y: 0 }
end

draw({ x: 10, y: 20 })
const line: Line = { start: { x: 0, y: 0 }, end: { x: 5, y: 5 } }
```

Regras:

- Campos sempre nomeados; construção posicional não existe.
- Todos os campos precisam aparecer (não há default implícito).
- Sem tipo esperado: `type.anon_struct_type_unknown`.
- Campo que não existe: `type.anon_struct_field_mismatch`.
- Formas removidas: `Point(...)`, `.{ ... }`, `(field: v)` →
  `parse.removed_struct_call_literal` / `type.struct_literal_named_fields_required`.
- **Nota 0.3.8:** o literal vazio `{ }` / `Point { }` não é aceito; ele é lido
  como mapa vazio. Para um valor padrão, implemente `core.Default` e chame
  `Point.default()`:

```ori
import ori.core = core

apply Point use core.Default
    default() -> Point
        return Point { x: 0, y: 0 }
    end
end

const p: Point = Point.default()
```

### 8.13 Atualização de struct (`with`)

```ori
const updated: Config = original with {
    verbose: true,
    timeout: 60,
} end
```

- Campos não citados mantêm o valor original.
- Produz um **novo** valor; `original` não muda.
- `end` fecha a expressão.

### 8.14 Variantes de enum

```ori
const d: Direction = Direction.North
const s: Shape = Shape.Circle(radius: 10.0)

-- forma curta, quando o tipo é conhecido pelo contexto
const d2: Direction = .North
const s2: Shape = .Circle(radius: 5.0)
```

- Payload sempre nomeado: `Shape.Rectangle(width: 5.0, height: 3.0)`.
- Construção de variante usa **parênteses**, não chaves. O `{ }` é reservado a
  struct.
- O `match` continua usando a variante sem ponto (seção 7.7).

### 8.15 Literais de coleção

```ori
const names: list[string] = ["Ana", "Bo", "Cara"]
const ages: map[int, int] = {1: 31, 2: 25}
const ids: set[int] = set{1, 2, 3}
const pair: tuple[int, string] = (1, "one")
const pair2: tuple[int, string] = tuple(2, "two")
```

- Mapa: chave é **literal** antes dos dois-pontos (`{ "k": v }`); identificador
  antes dos dois-pontos é campo de struct.
- Coleção vazia precisa de anotação: `const xs: list[int] = []`,
  `const m: map[int, int] = {}`, `const s: set[int] = set{}`.
- `tuple(a, b)` e `(a, b)` são equivalentes; tuplas têm no mínimo dois
  elementos.

### 8.16 Range

```ori
0..9       -- range[int] inclusivo
5..3       -- decrescente
```

Extremos precisam ser `int`. Ranges de `float` não existem.

### 8.17 `is`

```ori
describe(s: any[Shape]) -> string
    if s is Circle
        return "circle"
    end
    return "other"
end
```

Produz `bool`. Não faz narrowing e não testa variante de enum.

### 8.18 Ordem de avaliação

Expressões avaliam da esquerda para a direita, em profundidade. `and`/`or` pulam
o operando direito quando o resultado já está decidido. Efeitos colaterais
acontecem na ordem em que aparecem no texto.

---

## 9. Structs e enums

### 9.1 Structs

```ori
struct Rectangle
    width: int if it > 0
    height: int if it > 0

    area(self) -> int
        return self.width * self.height
    end

    scale(self, factor: int) -> Rectangle
        return Rectangle {
            width: self.width * factor,
            height: self.height * factor,
        }
    end
end
```

- Struct é **value type**: atribuir copia todos os campos.
- Campos gerenciados (`string`, coleções) copiam a referência.
- Contratos `if it …` são checados na **construção** e na **mutação**; violação
  é panic `contract.field_violation`.
- Métodos inerentes ficam no corpo do struct (seção 6.5).

### 9.2 Enums

```ori
enum Shape
    Circle(radius: float)
    Rectangle(width: float, height: float)
    Point
end
```

- Variantes unitárias (`Point`) ou com payload nomeado.
- Enum é value type.
- Payload posicional não existe.
- Discriminação sempre por `match` (não por `is`).

### 9.3 `@repr("C")` e `@align(N)`

```ori
@repr("C")
@align(16)
public struct UniformData
    matrix: int
    offset: int
end
```

- `@repr("C")` seleciona layout compatível com C (só a string exata `"C"`).
- `@align(N)` força alinhamento mínimo de `N` bytes (potência de 2 entre 1 e
  64), refletido em `mem.align_of` e nos headers C gerados.

---

## 10. Traits e `apply`

### 10.1 Declaração

```ori
trait Drawable
    draw(self, canvas: Canvas)
end

trait Serializable
    serialize(self) -> bytes
    deserialize(raw: bytes) -> result[Self, string]
end
```

- Método **sem corpo** é obrigatório.
- Método **com corpo** é implementação padrão (não existe palavra `default`).
- `Self` refere-se ao tipo que aplica a trait.
- Trait em posição de parâmetro (`p: Trait`) é despacho dinâmico, igual a
  `any[Trait]`.

### 10.2 `apply` — as formas aceitas

**Forma canônica (cabeçalho com dois-pontos, uma ou várias traits):**

```ori
apply Circle: Drawable
    draw(self, canvas: Canvas)
        canvas.draw_circle(self.center, self.radius)
    end
end

apply Player: Named, Scored
    name(self) -> string
        return self.tag
    end

    score(self) -> int
        return self.points
    end
end
```

**Forma compacta S3 (`apply Tipo use Trait`):**

```ori
apply Circle use Drawable
    draw(self, canvas: Canvas)
        canvas.draw_circle(self.center, self.radius)
    end
end
```

**Forma aninhada (várias seções `use`, cada uma com seus tipos associados):**

```ori
apply Circle
    use Drawable
        draw(self, canvas: Canvas)
            canvas.draw_circle(self.center, self.radius)
        end
    end

    use Sized
        size(self) -> int
            return 1
        end
    end
end
```

Regras:

- `use` na **mesma linha** do `apply` abre a forma compacta; na linha seguinte,
  abre a forma aninhada.
- Bloco aninhado com um único `use` e nada mais é rejeitado
  (`apply.redundant_use_block`).
- Ordem: métodos livres/binds → seções `use` → dentro de cada `use`: aliases de
  tipo, depois slots.
- Formas removidas: `implement Trait for Tipo` (`parse.implement_removed`) e
  `apply Trait to Tipo` (`parse.apply_trait_to_removed`).

### 10.3 Métodos inerentes vs trait

```ori
struct Counter
    value: int

    bump(self) -> int          -- inerente, não vem de trait
        return self.value + 1
    end
end
```

Métodos inerentes vivem no `struct`. `apply Tipo` com métodos livres continua
aceito por compatibilidade, mas `apply` deve ser usado principalmente para
conformidade com trait.

### 10.4 Tipos associados

```ori
apply Bag use Container
    alias Item = string

    first_item(self) -> Item
        return self.label
    end
end
```

- Escrito com `alias`, dentro de uma seção `use`.
- Transparente: `Item` **é** `string`.
- Escopo restrito àquela seção `use`; fora dela é `type.undefined_name`.
- Não é tipo associado abstrato de Rust/Swift: a trait declara assinatura
  concreta.
- `type Nome = …` foi removido (`parse.associated_type_keyword_removed`).

### 10.5 Bind: `slot = funçãoLivre`

```ori
compare_points(a: Point, b: Point) -> int
    return a.x - b.x
end

apply Point use Comparable
    compare = compare_points
end
```

A função livre recebe o valor como primeiro argumento. Não é atribuição de
runtime: é provisão de método em tempo de compilação.

### 10.6 Funções associadas (sem `self`)

```ori
apply User use core.Default
    default() -> User
        return User { name: "?", age: 0 }
    end
end

apply User
    make_empty() -> User
        return User { name: "", age: 0 }
    end
end

const a: User = User.default()
const b: User = User.make_empty()
```

- Sem `self`, o método é **função associada** e é chamado pelo nome do tipo.
- Usar `self` dentro dela é `bind.self_outside_method`.
- Chamar na instância (`u.default()`) é `type.assoc_fn_instance_call`.
- Funções associadas de trait não entram em `any[Trait]` (não há receptor).
- Despacho estático por bound:

```ori
make_default for T: core.Default (prototype: T) -> T
    return T.default()
end
```

Se dois bounds declararem `default`, a chamada `T.default()` é ambígua
(`type.ambiguous_method`).

### 10.7 `mut` em traits

```ori
trait Counter
    mut increment(self, by: int)
    mut reset(self)
    total(self) -> int
end
```

`mut` na trait exige `mut` na implementação. Método sem `mut` não altera campos
de `self`.

### 10.8 Traits genéricas

```ori
trait Container[Item]
    first(self) -> Item
end

apply Circle use Container[float]
    first(self) -> float
        return self.radius
    end
end
```

- Os argumentos são posicionais e obrigatórios (`use Container` sem argumento é
  `impl.trait_args_missing`).
- A contagem precisa bater (`impl.trait_arg_count_mismatch`).
- A assinatura é checada contra a trait **instanciada**
  (`impl.wrong_signature`).

### 10.9 Resolução de métodos

- Métodos inerentes resolvem pelo caminho `namespace.Tipo.metodo`.
- Métodos de trait resolvem pela tabela de implementações criada nos `use`.
- Nome ambíguo entre traits exige qualificação `Trait.metodo(receptor)`.

### 10.10 Traits da `ori.core`

| Trait | Método | Usada por |
|---|---|---|
| `Displayable` | `display(self) -> string` | `string(v)` e interpolação `f"…"` |
| `Equatable` | `equals(self, other: Self) -> bool` | `==`, `!=` |
| `Comparable` | `compare(self, other: Self) -> int` | `<`, `<=`, `>`, `>=`, ordenação |
| `Hashable` | `hash(self) -> int` (opcional) | chaves de `map`/`set`/`hash_table` |
| `Addable`/`Subtractable`/`Multiplicable`/`Divisible` | métodos aritméticos | `+ - * /` |
| `Default` | `default() -> Self` (associada) | valor padrão explícito |
| `Disposable` | `mut dispose(self)` | `using` |
| `Destructor` | `mut destroy(self) -> void` | limpeza na última referência |
| `Error` | `message(self) -> string` | tipos de erro |
| `Cloneable` | `clone(self) -> Self` | cópia explícita |
| `Iterable` | `mut next() -> optional[T]` (estrutural) | `for` |
| `Transferable` | marcador | travessia de task/canal |

`Iterable` e `Transferable` são **marcadores**: a checagem é estrutural ou
feita pelo checker, não pela tabela de métodos. `Hashable` é a exceção que
registra `hash` opcional.

---

## 11. Genéricos

### 11.1 Funções genéricas

```ori
identity[T](value: T) -> T
    return value
end

first[T](items: list[T]) -> optional[T]
    if ori.list.len(items) == 0
        return none
    end
    return some(items[0])
end
```

Parametrização por `[T]` depois do nome; vários parâmetros: `[K, V]`.

### 11.2 Bounds (`for T: Trait`)

```ori
import ori.core = core

max_of for T: core.Comparable (a: T, b: T) -> T
    if a.compare(b) >= 0
        return a
    end
    return b
end
```

- Lista `[T]` e cláusula `for` são **exclusivas**.
- Com `for`, todo parâmetro de tipo precisa ser declarado nela:

```ori
pick for K: core.Comparable, V: core.Displayable (k: K, v: V) -> K
    return k
end
```

- Vários bounds para o mesmo tipo são permitidos (`K: Hashable, K: Comparable`).

### 11.3 Bounds negativos

```ori
import ori.core = core

raw_copy for T: not core.Disposable (src: T) -> T
    return src
end
```

Impede chamada com tipos gerenciados/recursos. Violação:
`generic.constraint_not_satisfied`.

### 11.4 Structs, enums e aliases genéricos

```ori
struct Pair[A, B]
    first: A
    second: B
end

enum Either[Left, Right]
    Left(value: Left)
    Right(value: Right)
end

alias IntMap[V] = map[int, V]
alias Callback[T] = func(T) -> bool
```

### 11.5 Inferência no call site

```ori
const n: int = identity(42)                 -- T = int
const name: optional[string] = first(["Ada", "Bo"])
const empty: optional[int] = first[int]([]) -- explícito quando ambíguo
```

### 11.6 Monomorfização

Cada combinação concreta de argumentos gera uma implementação própria:

```text
identity(42)      -> identity_int
identity("hello") -> identity_string
```

Consequências: zero custo em runtime, otimização por tipo concreto, binário
maior e compilação mais longa quando há muitas instanciações. Instanciação
genérica circular é erro de compilação.

### 11.7 Genéricos de constante (const generics)

```ori
struct Buffer[const size: int]
    used: int
end

const words: int = 4
const small: Buffer[size: 8] = Buffer { used: 0 }
const computed: Buffer[size: words * 2] = Buffer { used: 0 }
```

- Argumentos são **nomeados**: `Buffer[size: 8]`, não `Buffer[8]`.
- O valor é parte da identidade do tipo: `Buffer[size: 8]` ≠ `Buffer[size: 16]`.
- CT-0 aceita literais inteiros, constantes inteiras/booleanas de módulo,
  aritmética inteira checada, comparações, lógica booleana e `if` inline.
- CT-0 **não** chama função, não aloca, não faz I/O, não lê ambiente nem FFI.
- Overflow, divisão por zero, ciclo de constantes e valor não inteiro são erros
  de compilação.
- Um parâmetro `const` pode ser repassado direto (`size: cap`), mas aritmética
  simbólica (`size: cap + 1`) ainda não faz parte do contrato.
- O valor só serve como etiqueta de compilação: não ocupa armazenamento.

### 11.8 Limites de v1

| Recurso | Situação |
|---|---|
| Tipos de tipo superior (HKT) | fora de escopo por decisão; `trait Functor[F[_]]` parseia mas não é implementável |
| Parâmetros de tipo variádicos | não existem (`tuple[T...]` é erro) |
| Tipo associado abstrato na trait | não existe; `alias` é do implementador |
| Inferência global (HM) | permanentemente fora de escopo |

---

## 12. Erros, ausência e panic

### 12.1 O modelo

| Construção | Uso | Propagação |
|---|---|---|
| `optional[T]` | valor pode estar ausente | `try`, `if some`, `match` |
| `result[T, E]` | operação pode falhar | `try`, `if ok`/`if err`, `match` |

Não há exceções, `throw`, `catch`, `null` nem `success`/`error`.

### 12.2 Formas de tratar

```ori
-- padrão explícito
match find_user(42)
    case some(user):
        greet(user)
    case none:
        io.println("not found")
end

-- binding de um ramo
if some(user) = find_user(42)
    greet(user)
end

-- valor padrão
const name: string = find_name(id).or("Anônimo")

-- propagação
const user: User = try find_user(id)
const user2: User = find_user(id).or_return()

-- contexto em erro string
const cfg: Config = try read_config(path).or_wrap("carregando configuração")
```

### 12.3 Tipos de erro

Qualquer tipo pode ser o `E` de `result[T, E]`. `ori.core.Error` define o
contrato:

```ori
trait Error
    message(self) -> string
end
```

```ori
import ori.core = core

struct ValidationError
    field: string
    reason: string
end

apply ValidationError use core.Error
    message(self) -> string
        return f"validation failed on '{self.field}': {self.reason}"
    end
end
```

Regras: a trait precisa estar importada; `message` é obrigatório
(`impl.missing_method`); assinatura diferente é `impl.wrong_signature`.
Tanto `apply T: core.Error` quanto `apply T use core.Error` são aceitos.

**União de erros com enum:**

```ori
import ori.core = core

enum AppError
    Network(cause: NetworkError)
    Validation(cause: ValidationError)
    Parse(cause: ParseError)
end

apply AppError use core.Error
    message(self) -> string
        match self
            case Network(cause):
                return cause.message()
            case Validation(cause):
                return cause.message()
            case Parse(cause):
                return cause.message()
        end
    end
end
```

`cause` é o nome usado no payload porque `error` é palavra reservada (assim como
`success`, `some`, `with`, `then`, `lazy`, `handle`, `attr`).

Há também `ori.Error`, um struct concreto com `code` e `message`:

```ori
import ori.Error = StdError

const e: StdError = StdError { code: "E_IO", message: "could not read file" }
```

### 12.4 Panic

Fontes de panic não recuperável:

- `panic("mensagem")`;
- `check condição` que falha;
- `todo()` e `unreachable()`;
- divisão inteira por zero;
- índice fora do intervalo;
- violação de contrato de campo/parâmetro;
- contagem de deslocamento inválida.

Panic não é capturável na linguagem. Use `result[T, E]` para falha esperada.

### 12.5 Padrões comuns

```ori
process(path: string) -> result[Output, string]
    const raw: string = try ori.fs.read_text(path)
    const parsed: Input = try parse(raw)
    const output: Output = try transform(parsed)
    return ok(output)
end

find_display_name(id: int) -> optional[string]
    const user: User = try find_user(id)
    const profile: Profile = try find_profile(user.profile_id)
    return some(profile.display_name)
end
```

Converter tipo de erro é explícito hoje:

```ori
run(path: string) -> result[void, AppError]
    match read_config(path)
        case ok(c):
            apply_config(c)
            return ok()
        case err(msg):
            return err(AppError.Parse(ParseError { message: msg }))
    end
end
```

---

## 13. Memória e limpeza

### 13.1 Semântica de valor

Todos os tipos têm semântica de valor por padrão. Atribuir um struct copia todos
os campos; não há aliasing entre o original e a cópia.

```ori
const a: Point = Point { x: 1, y: 2 }
const b: Point = a          -- cópia
```

### 13.2 Tipos gerenciados e aliasing

`string`, `bytes` e coleções (`list`, `map`, `set`, handles opacos) são
**gerenciados**: heap + contagem de referências. Atribuir copia a **referência**,
não os dados.

```ori
const a: list[int] = [1, 2, 3]
const b: list[int] = a
lists.push(a, 4)
-- b também tem 4: o mutador alterou o mesmo objeto no heap
```

| Operação | Efeito observável |
|---|---|
| `+`, fatiar, helpers `str.*` | produzem valor novo; entradas intactas |
| `lists.push`, `maps.set`, `sets.add`, atribuição por índice | alteram o objeto compartilhado **no lugar** |

`const` impede **rebinar o nome**, não mutar a coleção pela API. Copy-on-write
foi avaliado e **adiado**: mudaria o aliasing observável.

### 13.3 ARC e ciclos

O compilador insere retain/release em tempo de compilação. O runtime usa
contagem atômica, registra arestas fortes entre objetos gerenciados e roda um
coletor de ciclos por trial-deletion.

- Coleta cooperativa em pontos seguros, com limiar adaptativo (começa em 256
  alocações).
- `ori.test.live_allocations()`, `ori.test.collect_cycles()` e
  `ori.test.assert_no_leaks(label)` existem para testes.
- `ORI_TEST_LEAK_CHECK=1` faz `assert_no_leaks` falhar de forma ruidosa.

### 13.4 `using` + `Disposable`

Resumo (detalhes na seção 7.8): o tipo precisa implementar `core.Disposable`;
o `dispose()` roda em toda saída de escopo, em ordem LIFO.

### 13.5 `Destructor`

Struct ou enum pode optar por limpeza na última referência:

```ori
apply NativeHandle use core.Destructor
    mut destroy(self) -> void
        close_external(self.id)
    end
end
```

- Assinatura exigida: `mut destroy(self) -> void`.
- Roda **uma vez**, quando a vida ARC termina, antes de liberar payload e
  campos.
- Pode ler os campos; não pode fazer `self` escapar, ressuscitar, liberar campos
  manualmente nem fazer trabalho assíncrono.
- Com ciclos, o momento pode atrasar. Para fechar recurso em um escopo exato,
  use `using`.

### 13.6 Regiões (arenas)

```ori
import ori.mem = mem

main()
    using r: mem.Region = mem.region()
    -- alocações temporárias de frame
    mem.reset(r)      -- reset O(1)
end                   -- dispose determinístico na saída
```

- Alocação por bump, O(1), fora do lock global do ARC.
- `mem.reset(r)` descarta tudo de uma vez e preserva os blocos para o próximo
  ciclo.
- Uma `Region` não pode escapar do bloco por `return` (`using.escape`) nem
  atravessar threads/tasks (não é `Transferable`).

### 13.7 Sem gerenciamento manual

A linguagem não expõe aritmética de ponteiros, `malloc`/`free`, ponteiros crus
(fora de `extern c`) nem diretivas de alocação de pilha. O compilador decide
pilha vs heap; `@noalloc` verifica estaticamente que uma função não aloca.

### 13.8 Memória em FFI

- Valores gerenciados não atravessam `extern` como ponteiros crus sem conversão
  explícita.
- Memória alocada por C deve ser encapsulada em tipo com `Disposable` que chama
  a função C de liberação.
- A responsabilidade de segurança no limite FFI é do programador.

---

## 14. Concorrência

### 14.1 Modelo

O backend nativo expõe tarefas, canais, futures e atômicos como valores
tipados. Não há threads de SO cruas nem ponteiros crus na superfície.

### 14.2 `async` / `await`

```ori
import ori.io = io
import ori.task = task

async delayed_answer() -> int
    await task.sleep(1)
    return 42
end

async main()
    const answer: int = await delayed_answer()
    io.println(f"answer: {answer}")
end
```

- Chamar `async f(...) -> T` cria e devolve `future[T]` imediatamente.
- `await` só dentro de `async`; o operando precisa ser `future[T]`.
- Valores gerenciados vivos através de um `await` ficam retidos no frame.
- `using` é permitido e descarta em retorno normal, falha, cancelamento, `try`
  e saída de laço.
- Formas fora do subconjunto documentado falham com
  `backend.native_unsupported` antes de gerar código.

Subconjunto coberto: `await` dentro de `if`/`else`/`match`/`while`/`for` e
aninhamentos; `await` como statement; `const x: T = await f`; `return await f`;
`const x: T = try await f`; valores aguardados em retorno, argumentos,
operadores e condições.

### 14.3 `ori.task`

```text
task.spawn[T](work: func() -> T) -> task.Job[T]
task.join[T](job: task.Job[T]) -> result[T, task.JoinError]
task.detach[T](job: task.Job[T]) -> void
task.block_on[T](future: future[T]) -> T
task.sleep(ms: int) -> future[void]
task.create_token() -> task.CancelToken
task.cancel(token: task.CancelToken) -> void
task.is_cancelled(token: task.CancelToken) -> bool
task.associate(token: task.CancelToken, future: future[void]) -> void
```

- `spawn` roda função/closure sem argumento em thread nativa.
- Capturas e retorno precisam ser `Transferable`.
- `join` devolve `ok(valor)` ou `err(...)` (job já juntado, ausente ou panic).
- `detach` deixa a thread seguir sem join.
- `sleep` cria `future[void]` com timer não bloqueante.
- `block_on` é a ponte síncrona explícita.

### 14.4 Canais

```text
channel.create[T]() -> channel.Channel[T]
channel.create_bounded[T](capacity: int) -> optional[channel.Channel[T]]
channel.send[T](ch, value: T) -> result[void, channel.SendError]
channel.receive[T](ch) -> result[T, channel.ReceiveError]
channel.close[T](ch) -> void
```

- `create` é FIFO sem limite.
- `create_bounded` devolve `some` para capacidade positiva e `none` para zero ou
  negativa.
- Canal cheio faz `send` esperar; `close` acorda remetentes com erro.
- `receive` espera valor ou devolve erro quando fechado e vazio.
- Payload gerenciado precisa ser `Transferable`; o runtime retém e libera.

### 14.5 Atômicos

```text
atomic.new(value: int) -> atomic.AtomicInt
atomic.load(value) -> int
atomic.store(value, next: int) -> void
atomic.add(value, delta: int) -> int      -- devolve o novo valor
```

`AtomicInt` não é mutex de propósito geral. `Atomic[T]` genérico está adiado.

### 14.6 Cancelamento e escopos

```ori
import ori.cancel = cancel
import ori.list = lists
import ori.task = task

run_tasks()
    using scope: cancel.TaskScope = cancel.create_task_scope()
    const job: task.Job[void] = task.spawn(() => work())
    lists.push(scope.jobs, job)
    -- na saída: cancela o token e junta os jobs rastreados
end
```

| API | Papel |
|---|---|
| `cancel.create_scope()` | `CancelScope` com token novo |
| `cancel.create_scope_with(token)` | envolve token existente |
| `cancel.cancel(scope)` | cancela o token |
| `cancel.is_cancelled(scope)` | consulta |
| `cancel.defer_cancel(scope, ms)` | espera e cancela (futuro) |
| `cancel.create_task_scope()` | `TaskScope` com token + lista de jobs |
| `cancel.join_tasks(scope)` | junta os jobs explicitamente |

Ambos implementam `core.Disposable`: com `using`, o `dispose` roda na saída do
bloco. `CancelScope` cancela o token; `TaskScope` cancela e junta os filhos.

### 14.7 O que pode atravessar task/canal

`Transferable` cobre:

- escalares;
- `string`, `bytes`;
- `list[T]`, `map[K,V]`, `set[T]`, `optional[T]`, `result[T,E]`, tuplas;
- `future[T]`, `task.Job[T]`, `channel.Channel[T]` e handles opacos de coleção
  quando o conteúdo é transferível;
- structs com todos os campos transferíveis;
- enums com todos os payloads transferíveis;
- `atomic.AtomicInt` e handles de erro de task/canal.

**Não** são transferíveis: `fs.File`, `io.Input`, `io.Output`,
`net.Connection`, `net.Listener`, `net.UdpSocket`, `slice[T]`, `lazy[T]`,
`handle[T]` e `any[Trait]`. `task.CancelToken` é a exceção deliberada (só um
flag atômico).

Closure passada a `task.spawn` não pode capturar `var` lexical nem ler/escrever
`var` mutável de topo (`concurrency.global_mutable_capture`). `var` de módulo é
compartilhada sem sincronização: use `AtomicInt`, canal ou estado de dono único.

---

## 15. Atributos e compilação condicional

### 15.1 Lista fechada

| Atributo | Aplica a | Validação | Efeito atual |
|---|---|---|---|
| `@test` | função | sem argumentos; sem parâmetros, sem tipos e sem retorno | descoberta por `ori test` |
| `@deprecated("msg")` | qualquer declaração | exatamente uma string | aviso `attr.deprecated` nos usos |
| `@inline` | função | sem argumentos | validado; backend atual não age |
| `@no_inline` | função | sem argumentos | validado; backend atual não age |
| `@noalloc` | função | sem argumentos | verificação estática de não-alocação |
| `@align(N)` | struct | potência de 2 entre 1 e 64 | alinhamento mínimo no layout e no header C |
| `@repr("C")` | struct | exatamente `"C"` | layout compatível com C |
| `@c_export` / `@c_export("nome")` | função pública | sem argumento ou identificador C | exporta símbolo para host C |
| `@cfg(predicado)` | declaração de topo | um predicado estruturado | remove a declaração inativa |

Atributo desconhecido é `attr.unknown`; argumento inválido é `attr.invalid_arg`.

### 15.2 `@noalloc`

```ori
@noalloc
pure_add(a: int, b: int) -> int
    return a + b
end
```

Rejeita: alocação de heap, literais de coleção, formatação de string, closures,
`await`, `using`, iteração dinâmica de coleção e chamada a função não marcada
`@noalloc`. Violação: `perf.allocation_in_noalloc` (ex.:
`list literal allocates heap memory`).

### 15.3 `@cfg`

```ori
@cfg(target_os: linux)
linux_api() -> int
    return 1
end

@cfg(all(target_family: unix, feature: tls))
secure_api() -> int
    return 2
end

@cfg(any(target_os: linux, target_os: macos))
desktop_unix_api() -> int
    return 3
end

@cfg(not(execution_profile: embedded))
standalone_only() -> int
    return 4
end
```

Gramática fechada:

```text
predicado = chave ":" valor
          | "all" "(" predicado { "," predicado } [ "," ] ")"
          | "any" "(" ... ")"
          | "not" "(" predicado [ "," ] ")"
chave     = target_os | target_arch | target_family | execution_profile | feature
```

- `all`/`any` exigem ao menos um predicado; `not` exige exatamente um.
- `feature` precisa estar declarada em `[features]` do manifesto.

Valores aceitos:

| Chave | Valores |
|---|---|
| `target_family` | `unix`, `windows`, `wasm`, `none` |
| `target_os` | `linux`, `windows`, `macos`, `android`, `ios`, `freebsd`, `netbsd`, `openbsd`, `dragonfly`, `solaris`, `illumos`, `haiku`, `aix`, `emscripten`, `wasi`, `visionos`, `tvos`, `watchos`, `none` |
| `target_arch` | `x86`, `x86_64`, `arm`, `aarch64`, `mips`, `mips64`, `powerpc`, `powerpc64`, `riscv32`, `riscv64`, `s390x`, `sparc`, `sparc64`, `wasm32`, `wasm64`, `loongarch64`, `bpf`, `csky`, `hexagon`, `m68k`, `nvptx64`, `avr`, `msp430`, `xtensa` |
| `execution_profile` | `standalone`, `embedded` |
- Valores de target são fechados e checados; OS desconhecido não cai em `none`.
- Sintaxe do arquivo inteiro é checada mesmo em código inativo; resolução de
  nomes e tipos não.
- Declaração inativa não define símbolo, não conflita e não entra no HIR, na
  documentação nem em backend algum.
- **Só declarações de topo** aceitam `@cfg` em v1. Import, campo, parâmetro,
  statement e expressão não aceitam.
- `@cfg` é seleção de build, não fronteira de segurança/capacidade.

---

## 16. FFI e ABI nativa

### 16.1 Importar símbolo C (`extern`)

```ori
extern "C"
    public abs_int(x: int) -> int
    public var errno: int
end

main()
    const r: int = abs_int(-2)
end
```

- O rótulo de ABI aceita `"C"`/`c` e `"host"`/`host`.
- Convenção de chamada C, nome de símbolo exato (sem mangling).
- Parâmetros gerenciados (`string`, coleções, `any[Trait]`) são rejeitados com
  `extern.managed_type_in_ffi`; use escalares, `handle[T]` ou tipos com
  representação acordada.
- `public var` declara variável global do host.
- Prefira os wrappers públicos da stdlib a chamar símbolos do runtime direto.

### 16.2 Exportar função (`@c_export`)

```ori
module app.embed_add

@c_export
public add_scores(a: int, b: int) -> int
    return a + b
end

@c_export("mul_scores")
public mul(a: int, b: int) -> int
    return a * b
end
```

```bash
ori compile --lib examples/embed/add_scores.orl -o libadd_scores.so
```

Gera a biblioteca e um header C irmão. O host precisa chamar `ori_rt_init()`
antes e `ori_rt_shutdown()` no fim; `ori_rt_version()` e
`ori_rt_abi_version()` devolvem strings emprestadas (não liberar).

### 16.3 Tipos aceitos no limite ABI-1

Aceitos:

- escalares inteiros/float, `bool`, `void`, `string` terminada em NUL;
- `bytes` via `OriBytes { data, len }` (comprimento exato, inclusive NUL
  interno);
- structs não vazias, não genéricas, de campos escalares (ponteiro/out);
- structs gerenciadas via handle ARC opaco;
- pontes diretas de `optional` e `result` sobre payloads aceitos.

Rejeitados: `list`, `map`, `set`, `tuple`, soma aninhada, struct genérica,
struct vazia. Uma coleção pode viver **dentro** de struct gerenciada exposta
como handle opaco.

Parâmetros gerenciados são **emprestados**; valores gerenciados devolvidos
transferem uma referência e o host libera com `ori_arc_release`. `string`
devolvida usa ponteiro direto; `bytes` devolvida usa `OriBytes *out`.

### 16.4 Layout C e alinhamento

`@repr("C")` + `@align(N)` documentam o layout para buffers de GPU, headers C e
extensões nativas:

```ori
@repr("C")
@align(16)
public struct UniformData
    matrix: int
    offset: int
end
```

### 16.5 Dependências nativas declarativas

Em `ori.pkg.toml`, sem scripts de linker manuais:

```toml
[native.dependencies.raylib]
pkg_config = "raylib"
static = true
version = ">= 5.0"

[native.linux]
libraries = ["GL", "X11", "m", "dl"]
library_dirs = ["/usr/local/lib"]
link_flags = ["-Wl,-rpath,/opt/engine/lib"]

[native.macos]
frameworks = ["OpenGL", "Cocoa", "IOKit"]
```

---

## 17. Biblioteca padrão

### 17.1 Convenções

- Módulos públicos: `ori.X` (um nome por domínio).
- Imports explícitos: `import ori.io = io`.
- Caminhos antigos `ori.X.utils` / `ori.X.algorithms` continuam aceitos como
  compatibilidade silenciosa.
- **Coleções não têm métodos.** Use `lists.push(xs, v)`, não `xs.push(v)`.
- Funções globais embutidas: `len(texto)`, `string(v)`, `int(v)`, `float(v)`.
  Não existem `u8(v)`, `i8(v)`, `u16(v)`, `u32(v)`, `u64(v)`; valores estreitos
  vêm de literal com sufixo (`42u8`), `bytes.get` ou bit a bit.
- `len` é **só para `string`** (contagem de escalares Unicode). Para coleções,
  use `lists.len`, `maps.len`, `sets.len`, `bytes.len`.
- `io.print`/`io.println` recebem **`string`**. Converta antes:
  `io.println(string(42))`.

### 17.2 Mapa de módulos

| Grupo | Módulos |
|---|---|
| Núcleo | `ori.core`, `ori.Error`, `ori.mem`, `ori.convert`, `ori.handle`, `ori.lazy` |
| I/O e sistema | `ori.io`, `ori.fs` (alias `ori.files`), `ori.path`, `ori.os`, `ori.process`, `ori.args`, `ori.config`, `ori.log`, `ori.err_trace` |
| Texto e dados | `ori.string`, `ori.string_view`, `ori.bytes`, `ori.buffer`, `ori.slice`, `ori.span`, `ori.format`, `ori.json` |
| Coleções | `ori.list`, `ori.map`, `ori.set`, `ori.iter`, `ori.deque`, `ori.queue`, `ori.stack`, `ori.linked_list`, `ori.doubly_linked_list`, `ori.hash_table`, `ori.tree`, `ori.graph`, `ori.heap`, `ori.slotmap` |
| Numérico e tempo | `ori.math` (com `math.vec2`, `math.vec3`, `math.mat3`), `ori.random`, `ori.time` |
| Concorrência | `ori.concurrent`, `ori.task`, `ori.channel`, `ori.atomic`, `ori.cancel` |
| Rede | `ori.net`, `ori.net.http` |
| Outros | `ori.crypto`, `ori.validate`, `ori.test`, `ori.image` |

### 17.3 `ori.io`

```text
print(value: string)            -> void
println(value: string)          -> void
eprint(value: string)           -> void
eprintln(value: string)         -> void
read_line()                     -> optional[string]

stdin() / stdout() / stderr()   -> io.Input / io.Output
read(input, max_bytes)          -> result[optional[bytes], string]
write(output, data: bytes)      -> result[int, string]
flush(output)                   -> result[void, string]
close_input(input) / close_output(output) -> void
read_text(input, max_chars)     -> result[optional[string], string]
write_text(output, text)        -> result[int, string]
```

`read_line` devolve `none` no fim da entrada ou quando a linha não é UTF-8
válido.

### 17.4 `ori.fs`

```text
read_text(path) -> result[string, string]
write_text(path, content) -> result[string, string]
read_bytes(path) -> result[bytes, string]
write_bytes(path, content) -> result[string, string]
read_all(path) -> result[string, string]
append_text(path, content) -> result[void, string]
exists(path) -> result[bool, string]
delete(path) -> result[void, string]
list_dir(path) -> result[list[string], string]
create_dir(path) / create_dir_all(path) -> result[void, string]
is_file(path) / is_dir(path) -> result[bool, string]
copy(from, to) / rename(from, to) -> result[void, string]
read_text_async(path) -> future[result[string, string]]
write_text_async(path, content) -> future[result[string, string]]

open_read(path)  -> result[fs.File, string]
open_write(path) -> result[fs.File, string]
read(file, bytes_count) -> result[bytes, string]
write(file, data) -> result[int, string]
close(file) -> void
```

Helpers `.orl` adicionais:

```text
read_text_or(path, fallback) -> string
write_text_result(path, content) -> result[string, string]
exists_result(path) -> result[bool, string]
remove_file(path) -> result[void, string]
move_path(from, to) -> result[void, string]
try_read_text(path) -> result[string, FsError]
```

`FsError` tem as variantes `NotFound`, `PermissionDenied`, `AlreadyExists`,
`InvalidPath` e `Other(message: string)`.

`File` é handle opaco; use `using` ou `close`. `read_bytes` preserva NUL
interno.

### 17.5 `ori.string`

```text
len(s) / concat(a, b) / split(s, sep) / contains(s, sub)
starts_with(s, p) / ends_with(s, p)
trim(s) / trim_start(s) / trim_end(s)
to_upper(s) / to_lower(s) / replace(s, from, to)
slice(s, start, end) / chars(s) / to_bytes(s) / from_bytes(b)
parse_int(s) -> result[int, string]
parse_float(s) -> result[float, string]
```

Helpers `.orl`: `is_empty`, `blank`, `replicate`, `replace_all`,
`join_non_empty`, `truncate`.

Posições usam **valores escalares Unicode**, não offsets de byte. Segmentação
de grafema não é API.

### 17.6 `ori.bytes` e `ori.convert`

```text
bytes.len / concat / slice / to_hex / from_hex / decode_utf8 / get
conv.float_to_string / bool_to_string
conv.string_to_int  -> optional[int]
conv.string_to_float -> optional[float]
```

`bytes` não indexa com `[]`; use `bytes.get`.

### 17.7 `ori.list`, `ori.map`, `ori.set`

```text
lists.new / with_capacity / len / is_empty / capacity / clear
lists.push / pop / insert / remove / get / try_get / set
lists.first_or / last_or / get_or / index_of / contains
lists.slice / window / reverse / sort / clone / to_list

maps.new / set / get / try_get / contains / remove / try_remove
maps.len / is_empty / capacity / reserve / clear / clone
maps.from_entries / keys / values / entries

sets.new / add / contains / remove / try_remove
sets.len / is_empty / capacity / reserve / clear / clone
sets.to_list / from_list / union / intersection / difference
```

`maps.get(m, k)` devolve o valor direto; use `maps.try_get` quando a ausência
importa. `reserve` garante capacidade; `clear` mantém a capacidade alocada.

### 17.8 Estruturas de dados

`deque`, `queue`, `stack`, `linked_list`, `doubly_linked_list` expõem o mesmo
padrão: `new`, `push_*`, `pop_*`, `peek`, `len`, `is_empty`, `clear`, `clone`,
`to_list` (listas ligadas acrescentam cursores e `find`).

- Listas ligadas são indexadas sobre buffer duplo (sem nós ponteiro): fim O(1)
  amortizado; cursor/insert/remove O(n).
- `tree.Tree[T]` é arena opaca com `NodeId`; `tree.value` em id inválido é erro
  de runtime, prefira `try_value`.
- `hash_table.HashTable[K,V]` dá controle de capacidade e `get/remove` com
  `optional`.
- `graph.Graph[N]` é lista de adjacência; `graph.new(true)` dirigido,
  `graph.new(false)` não dirigido; BFS/DFS/topological/components/shortest_path.
- `heap.Heap[T]` é min-heap para `int`, `string` e tipos `Comparable`.
- `slotmap` e `span` são estruturas de índice/janela sobre buffer.
- Iteração `for` sobre handles opacos é por **snapshot**.

### 17.9 `ori.iter` (eager)

```text
map(values, mapper) / filter(values, predicate)
any / all / count_where
take / skip / reverse / reduce / find
flat_map / sort / sort_by / unique / zip / partition / group_by / flatten
```

Tudo é **eager**: devolve `list[R]` imediatamente. Para lazy de verdade use
`lazy[T]` ou um tipo com `core.Iterable`.

### 17.10 `ori.math`, `ori.random`, `ori.time`, `ori.format`

```text
math.abs / min / max / clamp / floor / ceil / round
math.sqrt / pow / log / log2 / sin / cos / tan
math.pi / e / infinity / nan / is_nan / is_infinite

random.int(min, max) / float(min, max) / bool()
random.choice(items) -> optional[T] / shuffle(items) -> list[T]

time.now() -> int (ms unix) / sleep(ms) / duration_ms(start, end)
time.Instant / Duration + instant_now, duration_seconds, elapsed_since, add, sub

format.number(v, decimals) / percent / hex / binary
format.date(millis, style) / datetime(millis, style, locale)
format.bytes_size(bytes, style)
```

Não há alargamento implícito `int`→`float` em chamada; use `float(v)`.

### 17.11 `ori.mem`

```text
mem.size_of(value) -> int      -- constante de compilação
mem.align_of(value) -> int
mem.region() -> mem.Region
mem.reset(region) -> void
```

O argumento é testemunha de tipo; não existe `size_of[T]()`.

### 17.12 `ori.json`

```ori
enum Value
    Null
    Bool(value: bool)
    Number(value: float)
    String(value: string)
    Array(items: list[Value])
    Object(fields: map[string, Value])
end

json.parse(text) -> result[json.Value, string]
json.try_parse(text) -> result[json.Value, JsonError]
json.try_read(path) -> result[json.Value, JsonError]
json.stringify(value) -> string
json.stringify_pretty(value) -> string
```

### 17.13 `ori.test`

```ori
@test
test_addition()
    check 1 + 1 == 2
end

test.assert(condition: bool, message: string)
test.assert_eq[T](a: T, b: T)
test.assert_ne[T](a: T, b: T)
test.fail(message: string)
```

`ori test <arquivo-ou-projeto> --filter <nome>` roda só os testes cujo nome
contém o filtro. Testes `async` precisam ser sem parâmetros e retornar `void`;
o runner espera o future.

### 17.14 `ori.crypto`

```text
password_hash(password) -> string          -- argon2id, PHC
password_verify(password, encoded) -> bool
totp_generate_secret() -> string
totp_code(secret, unix_secs) -> string
totp_verify(secret, code, unix_secs, window) -> bool
try_hash_password / try_totp_generate_secret / try_totp_code
```

Nunca use MD5/SHA puro para senha.

### 17.15 `ori.net` e `ori.net.http`

```text
net.connect / connect_async / connect_tls / connect_tls_async
net.listen / accept / accept_async / close_listener / listener_port
net.read_some / read_some_async / write_all / write_all_async
net.close / is_closed
net.udp_bind / udp_send_to / udp_recv_from / udp_close / udp_local_port
net.try_connect -> result[net.Connection, NetError]
```

I/O de rede síncrono é bloqueante; variantes `*_async` devolvem `future[…]` e
usam reator de prontidão (poll) para accept/read/write/UDP, e workers para
connect/TLS.

`ori.net.http` traz `Response`, `HttpError`, `get_plain`, `get_tls`,
`post_plain`, `post_tls`, `build_request`, `parse_response`.

### 17.16 `ori.os`, `ori.process`, `ori.args`, `ori.log`, `ori.config`

```text
os.args() -> list[string] / os.env(name) -> optional[string]
os.exit(code) / os.pid() / os.platform() / os.arch()

process.run(program, args) -> result[int, string]
process.run_output(program, args) -> result[ProcessOutput, string]
process.stdout_text / stderr_text -> result[string, string]

args.all() / count() / get_or(index, fallback) / program_name_or(fallback)

log.info / warn / error_message / debug

config.read_text / read_text_or / write_text / read_json / write_json
```

### 17.17 `ori.path`, `ori.validate`, `ori.err_trace`, `ori.image`

```text
path.join / normalize / is_absolute / is_relative
path.base_name / extension / name_without_extension / parent
path.has_extension / change_extension / absolute / relative

validate.positive / non_negative / negative / non_zero
validate.between / in_range / one_of / one_of_string
validate.not_empty / not_empty_text / blank
validate.min_length / max_length / length_between
validate.even / odd / all_digits / int_in_range

err_trace.push(file, line, message) -> string
err_trace.format(message) -> string

image.encode_ppm / write_ppm / encode_bmp / write_bmp
```

### 17.18 Módulos nativos menores

| Módulo | Conteúdo |
|---|---|
| `ori.buffer` | `new`, `get`, `set`, `len`, `is_empty`, `fill`, `as_slice` |
| `ori.slice` | `len`, `get` (janela sobre `list`) |
| `ori.lazy` | `once`, `force`, `is_consumed` |
| `ori.handle` | `null`, `is_null` |
| `ori.cancel` | escopos de cancelamento e de tarefas |
| `ori.concurrent` | `copy[T]`, `transfer[T]` e variantes monomórficas legadas |
| `ori.err_trace` | rastro de erro com arquivo/linha |
| `ori.string_view` | `from_string`, `from_sub`, `len`, `subview`, `to_string` |

---

## 18. Projeto, build e ferramentas

### 18.1 Layout

```text
meu-projeto/
  ori.proj          -- obrigatório
  main.orl          -- entrada recomendada
  app/              -- domínios opcionais (.orl)
  docs/             -- sidecars .oridoc opcionais
```

`ori new` cria `ori.proj`, `main.orl` (ou `lib.orl` com `--lib`) e `docs/`. Não
impõe `src/`, `lib/` ou `bin/`.

### 18.2 `ori.proj`

```ini
manifest = 1
name = "demo"
version = "0.1.0"
kind = "app"
entry = "main.orl"

[source]
root_namespace = "app"

[dependencies]
demo.math = { path = "../math", version = "0.1.0" }

[features]
default = ["tls"]
tls = []
telemetry = []

[docs]
paths = ["docs"]
mode = "sidecar-first"
require_public = "off"
```

| Campo | Obrigatório | Papel |
|---|---|---|
| `manifest` | não | versão do formato (aceita `1`) |
| `name`, `version` | não | metadados |
| `kind` | não | `app` (padrão) ou `lib` |
| `entry` | **sim** | arquivo `.orl` de entrada |
| `source.root` | não | raiz do código (omitido = pasta do `ori.proj`) |
| `source.root_namespace` | não | prefixo de módulo esperado |
| `dependencies.<nome>` | não | dependência local ou git |
| `features.default` | não | features ligadas por padrão |
| `features.<nome>` | não | declara feature cfg (valor `[]`) |
| `docs.paths` / `docs.mode` / `docs.require_public` | não | documentação sidecar |

### 18.3 `ori.pkg.toml`

Manifesto de **pacote** distribuível (não substitui `ori.proj`):

```toml
[package]
name = "demo.app"
version = "0.1.0"
entry = "main.orl"
ori_version = "0.3.1"
description = "Demo app"

[dependencies]
demo.math = { path = "../demo-math", version = "0.1.0" }

[features]
default = []
tls = []

[native.dependencies.raylib]
pkg_config = "raylib"
static = true
version = ">= 5.0"
```

### 18.4 Lockfile

`ori lock` grava `ori.lock` ao lado do manifesto, com fonte/versão exata
(incluindo commit de dependência git). `ori lock --locked` valida sem reescrever.
Com lockfile presente, `check`, `build`, `compile` e `test` rejeitam manifesto
que resolva diferente do snapshot.

### 18.5 `.oridoc`

```text
oridoc 1

module app.kanban.board

doc load_board
  summary:
    Carrega o quadro.
  returns:
    `result[Board, string]`
end
```

Prioridade inline/sidecar vem de `[docs].mode` (`sidecar-first` por padrão).

### 18.6 CLI

| Comando | Uso |
|---|---|
| `ori new` / `ori init` | criar/inicializar projeto |
| `ori check` | checar tipos e diagnósticos |
| `ori build` / `ori compile` | gerar binário nativo (`--lib` para biblioteca) |
| `ori run` | compilar e executar (pode usar JIT) |
| `ori test` | rodar funções `@test` |
| `ori fmt` | formatar |
| `ori lint` | avisos de lint |
| `ori doc` | extrair docs (`file`, `check`, `export`) |
| `ori explain` | explicar um código de diagnóstico |
| `ori summary` | resumo do projeto (entrada, namespaces, imports) |
| `ori migrate-syntax` | reescrever sintaxe pré-S3 |
| `ori lex` / `ori parse` | depurar tokens/AST |
| `ori repl` | REPL sobre JIT nativo |
| `ori debug` | DAP (Debug Adapter Protocol) |
| `ori bindgen` | gerar bindings FFI a partir de header C |
| `ori daemon` | serviço JSON-RPC persistente |
| `ori doctor` | saúde de ambiente/stdlib/runtime |
| `ori install` / `get` / `lock` / `publish` | pacotes e registro |
| `ori emit` | artefatos de depuração |
| `ori update` | atualizar toolchain |

Flags globais: `--no-color`, `--target`, `--execution-profile`
(`standalone`/`embedded`), `--features`, `--no-default-features`.

---

## 19. Diagnósticos

### 19.1 Formato

```text
categoria.nome_especifico
```

Exemplo real:

```text
error[type.arg_type_mismatch]: argument 1 expects `string`, found `int`
  --> main.orl:6:16
   |
 6 |     io.println(42)
   |                ^^
   |                 this argument
   |
   = action: change the argument to produce `string`
```

Cada diagnóstico deve responder **o quê**, **onde** e **o que fazer**. Mensagens
não podem recomendar forma removida da linguagem.

### 19.2 Categorias emitidas

| Prefixo | Domínio | Exemplos |
|---|---|---|
| `lex` | léxico | `lex.unexpected_character`, `lex.unclosed_block_comment` |
| `parse` | gramática | `parse.func_removed`, `parse.else_if_removed`, `parse.chained_comparison` |
| `name` | nomes | `name.undefined`, `name.private` |
| `bind` | imports | `bind.unused_import`, `bind.import_not_found`, `bind.stdlib_module_unknown` |
| `type` | tipos | `type.type_mismatch`, `type.local_inference_failed`, `type.missing_return` |
| `generic` | genéricos | `generic.constraint_not_satisfied` |
| `impl` | traits | `impl.trait_not_found`, `impl.missing_method`, `impl.wrong_signature` |
| `apply` | blocos `apply` | `apply.redundant_use_block` |
| `match` | padrões | `match.non_exhaustive`, `match.or_pattern_binding` |
| `contract` | contratos | `contract.field_violation`, `contract.param_violation` |
| `using` | recursos | `using.not_disposable`, `using.escape` |
| `mut` | mutabilidade | atribuição a `const`, método `mut` em `const` (`mut.const_method_call`) |
| `async` | assíncrono | `async.await_outside_async` |
| `concurrency` | travessia | `concurrency.not_transferable`, `concurrency.global_mutable_capture` |
| `attr` | atributos | `attr.unknown`, `attr.invalid_arg`, `attr.invalid_test_signature` |
| `cfg` | compilação condicional | `cfg.target_invalid` |
| `extern` | FFI de entrada | `extern.managed_type_in_ffi` |
| `native` | backend/ABI | `native.runtime_symbol_missing` |
| `backend` | limites do backend | `backend.native_unsupported` |
| `consteval` | CT-0 | constantes inválidas em tipo |
| `perf` | desempenho estático | `perf.allocation_in_noalloc` |
| `project` | manifesto | layouts e campos inválidos |
| `doc` | documentação | `doc.param_name_mismatch` |
| `lint` | estilo | avisos de `ori lint` |
| `task` | runtime de tarefas | erros de join/cancelamento |

### 19.3 Ferramentas

```bash
ori explain match.non_exhaustive     # explica o código
ori check main.orl                   # checagem
ori lint .                           # avisos de estilo
ori doc check                        # valida docs inline e sidecar
```

O catálogo normativo é [`docs/spec/13-error-catalog.md`](../../spec/13-error-catalog.md).
Todo código emitido precisa de entrada no catálogo e teste negativo
(`cargo test -p ori-driver --test diagnostic_catalog`).

### 19.4 Erros comuns e correção

| Diagnóstico | Causa provável | Correção |
|---|---|---|
| `parse.module_missing` | arquivo sem `module` | adicione `module app.x` na 1ª linha |
| `parse.import_after_declaration` | import depois de declaração | mova imports para cima |
| `parse.func_removed` | `func nome()` | escreva `nome()` |
| `parse.else_if_removed` | `else if` | use `elif` |
| `parse.question_propagate_removed` | `expr?` | use `try expr` |
| `parse.removed_struct_call_literal` | `Tipo(...)` | use `Tipo { campo: v }` |
| `type.local_inference_failed` | RHS não óbvio | anote o tipo |
| `type.missing_return` | função não-void sem `return` | retorne em todos os caminhos |
| `type.arg_type_mismatch` | tipo errado em chamada | converta explicitamente |
| `type.unused_result` (aviso) | `result` descartado | use `try`, `match` ou `if ok` |
| `match.non_exhaustive` | falta caso | cubra variantes ou `case else:` |
| `impl.trait_not_found` | trait não importada | `import ori.core = core` |
| `using.not_disposable` | tipo sem `Disposable` | implemente a trait |
| `contract.field_violation` | contrato `if it` violado | corrija o valor na construção |
| `extern.managed_type_in_ffi` | `string` em `extern "C"` | use escalar/`handle[T]`/`@c_export` |

---

## 20. Estabilidade, compatibilidade e migração

### 20.1 Contrato estável do ciclo S3 (0.3.x)

- arquivos `.orl` em UTF-8;
- blocos terminados em `end` (rótulos opcionais);
- `module` obrigatório no topo;
- imports explícitos (`path (A)`, `path as alias`, `path`);
- tipos explícitos em bindings públicos, parâmetros e retornos públicos;
- compostos com `[]` (`optional[T]`, `result[T,E]`, `list[T]`, `array[T,N]`,
  `simd[T,N]`, …);
- ausência por `optional[T]`; falha por `result[T,E]`;
- propagação **só** por `try expr`;
- construção de struct por `Tipo { campo: valor }` ou `{ campo: valor }` com tipo
  esperado;
- enum por `Enum.Variant(...)` ou `.Variant(...)` com tipo esperado;
- traits por `apply Tipo: Trait` (canônico) ou `apply Tipo use Trait`;
- métodos inerentes dentro do `struct`;
- nome de trait nu em parâmetro como despacho dinâmico;
- backend nativo como referência semântica.

### 20.2 Formas adicionadas depois do corte S3

`newtype`, `match` como expressão, `case a or b:`, `if ok`/`if err`,
desestruturação de struct, `alias` em seção `use`, const generics com argumento
nomeado, `@align(N)`, `@noalloc`, `simd[T,N]`, `mem.region()`,
`apply Tipo: A, B` e `import path as alias`.

### 20.3 Experimental (pode mudar)

- formato final de pacote e lockfile;
- registro hospedado e limites do REPL;
- APIs de stdlib marcadas experimentais;
- detalhes de otimização genérica e tamanho de binário;
- detalhes finos da inferência local além da opção B;
- nomes finais de aliases de domínio da stdlib.

### 20.4 Lacunas conhecidas

| Lacuna | Detalhe |
|---|---|
| HKT | parseia, não implementa; fora de escopo |
| `@c_export` com coleções | `list`, `map`, `set`, tupla, soma aninhada e struct genérica não atravessam |
| Recursão infinita | não detectada em compilação; runtime reporta `stack overflow` |
| `bytes[i]` | indexação não existe |
| literal vazio `{ }` | não é struct literal em 0.3.8 |

### 20.5 Migração de sintaxe (pré-S3 → S3)

| Não escreva | Escreva | Diagnóstico |
|---|---|---|
| `namespace app.main` | `module app.main` | `parse.namespace_removed` |
| `func f(...)` | `f(...)` | `parse.func_removed` |
| `list of T`, `optional of T` | `list[T]`, `optional[T]` | `parse.removed_of_type` |
| `success(v)` / `error(e)` | `ok(v)` / `err(e)` | `parse.result_ctor_renamed` |
| `expr?` | `try expr` | `parse.question_propagate_removed` |
| `else if` | `elif` | `parse.else_if_removed` |
| `implement T for X` | `apply X: T` | `parse.implement_removed` |
| `apply T to X` | `apply X: T` | `parse.apply_trait_to_removed` |
| `do(x) => …` | `(x) => …` | `parse.do_removed` |
| `where T is Trait` | `for T: Trait` | `parse.removed_where_bound` |
| `import only path (A)` | `import path (A)` | `parse.import_only_removed` |
| `import alias = path` | `import path as alias` | `parse.expected_declaration` + `bind.import_not_found` |
| `Type(...)` | `Type { campo: v }` | `parse.removed_struct_call_literal` |
| `type Nome = T` (em `use`) | `alias Nome = T` | `parse.associated_type_keyword_removed` |
| `.{ campo: v }` | `{ campo: v }` | `parse.removed_struct_call_literal` |

Reescrita automática: `ori migrate-syntax <caminho>` (best-effort; revise o
diff).

### 20.6 Fatos de versão

| Item | Valor |
|---|---|
| Versão do projeto/compilador | `0.3.8` |
| Superfície | S3 (`0.3.0`) + inferência local B (`0.3.1`) |
| ABI nativa | `ori-native-abi-1` |
| Execução | AOT nativo; `ori run` pode usar JIT Cranelift |
| Implementação | Rust no `main` até 1.0; autohospedagem experimental fora do `main` (ADR-0006) |
| FREEZE-1 | fechado em 2026-07-19; nenhum bump para 0.4 |

---

## Apêndice A — Gramática resumida (não normativa)

Resumo de leitura. A gramática formal é
[`docs/spec/03-grammar.ebnf`](../../spec/03-grammar.ebnf); em conflito, vale a
especificação.

```ebnf
source_file   = module_decl { import_decl | imports_block } { top_level_decl } ;
module_decl   = "module" qualified_name ;
qualified_name= IDENT { "." IDENT } ;

import_decl   = [ "public" ] "import" import_form ;
import_form   = qualified_name "(" [ import_item { "," import_item } [ "," ] ] ")"
              | qualified_name ( "as" | "=" ) IDENT
              | qualified_name ;
import_item   = IDENT [ ( "as" | "=" ) IDENT ] ;
imports_block = "imports" { import_form { "," import_form } } "end" ;

top_level_decl= { attr } ( func_decl | struct_decl | enum_decl | trait_decl
              | apply_decl | alias_decl | newtype_decl | const_decl
              | var_decl | extern_block ) ;

attr          = "@" IDENT [ "(" [ attr_arg { "," attr_arg } ] ")" ] ;
attr_arg      = STRING | IDENT ":" ( STRING | IDENT ) ;

func_decl     = [ "public" ] [ "async" | "iter" ] [ "mut" ] IDENT
                [ type_params | bound_clause ]
                "(" [ param_list ] ")" [ "->" type ] func_body ;
func_body     = "=>" expression | block ;
type_params   = "[" type_param { "," type_param } "]" ;
type_param    = IDENT | "const" IDENT ":" type ;
bound_clause  = "for" bound { "," bound } ;
bound         = IDENT ":" [ "not" ] type_ref ;
param_list    = param { "," param } [ "," ] ;
param         = IDENT ":" type [ "=" expression ] [ "if" expression ] [ "..." ] ;

struct_decl   = [ "public" ] "struct" IDENT [ type_params ]
                { field_decl } { func_decl } "end" ;
field_decl    = IDENT ":" type [ "if" expression ] ;
enum_decl     = [ "public" ] "enum" IDENT [ type_params ]
                { IDENT [ "(" named_field { "," named_field } ")" ] } "end" ;
named_field   = IDENT ":" type ;

trait_decl    = [ "public" ] "trait" IDENT [ type_params ] { trait_member } "end" ;
trait_member  = func_signature | func_decl ;
func_signature= [ "public" ] [ "mut" ] IDENT "(" [ param_list ] ")" [ "->" type ] ;

apply_decl    = "apply" type_ref ":" type_ref { "," type_ref } { apply_member } "end"
              | "apply" type_ref [ "use" type_ref ] { apply_member } "end" ;
apply_member  = func_decl | IDENT "=" IDENT | "use" type_ref { apply_member } "end"
              | "alias" IDENT "=" type ;

alias_decl    = [ "public" ] "alias" IDENT [ type_params ] "=" type ;
newtype_decl  = [ "public" ] "newtype" IDENT "=" type ;
const_decl    = [ "public" ] "const" IDENT [ ":" type ] "=" expression ;
var_decl      = [ "public" ] "var" IDENT [ ":" type ] "=" expression ;

extern_block  = "extern" ( "c" | "host" | STRING ) { extern_member } "end" ;
extern_member = [ "public" ] IDENT "(" [ param_list ] ")" [ "->" type ]
              | [ "public" ] "var" IDENT ":" type ;

type          = primitive | qualified_name [ "[" type { "," type } "]" ]
              | "optional" "[" type "]" | "result" "[" type "," type "]"
              | "list" "[" type "]" | "map" "[" type "," type "]"
              | "set" "[" type "]" | "range" "[" type "]" | "lazy" "[" type "]"
              | "future" "[" type "]" | "handle" "[" type "]" | "any" "[" type_ref "]"
              | "slice" "[" type "]" | "buffer" "[" type "]"
              | "array" "[" type "," (INTEGER | "size:" type) "]"
              | "simd" "[" type "," (INTEGER | "lanes:" type) "]"
              | "func" "(" [ type { "," type } ] ")" [ "->" type ]
              | "tuple" "[" type "," type { "," type } "]" ;

block         = { statement } "end" [ IDENT ] ;
statement     = const_decl | var_decl | assignment | return_stmt | suspend_stmt
              | "break" | "continue" | if_stmt | if_bind_stmt | while_stmt
              | while_some_stmt | for_stmt | repeat_stmt | loop_stmt | match_stmt
              | using_stmt | check_stmt | expression ;
assignment    = lvalue ( "=" | "+=" | "-=" | "*=" | "/=" ) expression ;
lvalue        = IDENT { "." IDENT | "[" expression "]" } ;
return_stmt   = "return" [ expression ] ;
suspend_stmt  = "suspend" expression ;

if_stmt       = "if" expression { statement } { "elif" expression { statement } }
                [ "else" { statement } ] "end" [ "if" ] ;
if_bind_stmt  = "if" ( "some" | "ok" | "err" ) "(" IDENT ")" "=" expression
                { statement } [ "else" { statement } ] "end" ;
while_stmt    = "while" expression block ;
while_some_stmt = "while" "some" "(" IDENT ")" "=" expression block ;
for_stmt      = "for" IDENT [ "," IDENT ] "in" expression block ;
repeat_stmt   = "repeat" expression [ "times" ] block ;
loop_stmt     = "loop" block ;

match_stmt    = "match" expression { match_case } "end" [ "match" ] ;
match_case    = "case" pattern [ "if" expression ] ":" { statement }
              | "case" "else" ":" { statement } ;
pattern       = "_" | literal | IDENT | IDENT "(" [ pattern_arg { "," pattern_arg } ] ")"
              | "some" "(" pattern ")" | "none"
              | "ok" "(" pattern ")" | "err" "(" pattern ")"
              | "tuple" "(" pattern { "," pattern } ")"
              | pattern { "or" pattern } ;
pattern_arg   = pattern | IDENT ":" pattern ;

using_stmt    = "using" IDENT ":" type "=" expression ;
check_stmt    = "check" expression [ "," STRING ] ;

expression    = pipe_expr | closure | if_expr | struct_update ;
pipe_expr     = or_expr { "|>" or_expr } ;
or_expr       = and_expr { "or" and_expr } ;
and_expr      = bor_expr { "and" bor_expr } ;
bor_expr      = bxor_expr { "|" bxor_expr } ;
bxor_expr     = band_expr { "^" band_expr } ;
band_expr     = shift_expr { "&" shift_expr } ;
shift_expr    = compare_expr { ("<<" | ">>") compare_expr } ;
compare_expr  = add_expr [ compare_op add_expr ] ;
add_expr      = mul_expr { ("+" | "-") mul_expr } ;
mul_expr      = unary_expr { ("*" | "/" | "%") unary_expr } ;
unary_expr    = "-" unary_expr | "not" unary_expr | "~" unary_expr
              | "try" unary_expr | "await" unary_expr | postfix_expr ;
postfix_expr  = primary { "." ( IDENT | INTEGER ) | "(" [ arg_list ] ")"
              | "[" [ expression ] [ ".." [ expression ] ] "]" | IDENT } ;
arg_list      = arg { "," arg } [ "," ] ;
arg           = [ IDENT ":" ] expression | ".." expression ;

primary       = literal | IDENT | qualified_name | "self" | "(" expression ")"
              | "(" expression "," expression { "," expression } ")"
              | "[" [ expression { "," expression } [ "," ] ] "]"
              | [ type_ref ] "{" [ entry { "," entry } [ "," ] ] "}"
              | "set" "{" [ expression { "," expression } [ "," ] ] "}"
              | type_ref "." IDENT [ "(" [ entry_list ] ")" ]
              | "." IDENT [ "(" [ entry_list ] ")" ] ;

if_expr       = "if" expression "then" expression "else" expression ;
struct_update = expression "with" "{" [ entry { "," entry } [ "," ] ] "}" "end" ;
closure       = "(" [ closure_param { "," closure_param } ] ")" [ "->" type ]
                ( "=>" expression | block ) ;
closure_param = IDENT [ ":" type ] ;

entry         = IDENT ":" expression | literal ":" expression ;
literal       = INTEGER | FLOAT | STRING | FSTRING | TRIPLE_STRING | BYTES
              | "true" | "false" | "none" | expression ".." expression ;
```

---

## Apêndice B — Notas de fidelidade (0.3.8)

Observações feitas compilando os exemplos com `ori 0.3.8`. Elas não substituem a
especificação; servem para evitar surpresa ao consultar documentos antigos.

1. **`extern`** — `extern c`, `extern "C"`, `extern host` e `extern "host"` são
   aceitos. `extern c` com nome de módulo reservado (`module app.extern`) é erro
   de parse por causa do segmento reservado, não do `extern`.
2. **`using` exige tipo** — `using nome: Tipo = expr`; a forma sem anotação é
   erro de parse.
3. **Literal vazio de struct** — `{ }` e `Point { }` não são struct literal em
   0.3.8 (o primeiro vira mapa). Use `Point.default()` com `core.Default`.
4. **`bytes` não indexa** — `b[0]` é `type.not_indexable`; use `bytes.get`.
5. **Tuplas** — `(a, b)` e `tuple(a, b)` são aceitos como literal.
6. **`len` é só de string** — para listas use `lists.len(xs)`.
7. **`ori.fmt` não existe** — o módulo de formatação é `ori.format`.
8. **Trait genérica é aplicável** — `apply Tipo use Container[float]` compila,
   com os argumentos posicionais obrigatórios.
9. **`@test` é validado no runner** — `ori check` aceita assinatura inválida;
   `ori test` reporta `attr.invalid_test_signature`.
10. **Precedência** — comparação liga mais forte que bitwise; use parênteses em
    máscaras.
11. **`@noalloc`** — literais de lista, `f"…"`, closures, `await`, `using` e
    chamadas não marcadas são rejeitados com `perf.allocation_in_noalloc`.
12. **`arrays` e `simd`** — `array[T, N]` e `array[T, size: N]` são o mesmo
    tipo; `simd[T, N]` e `simd[T, lanes: N]` também.
13. **Conversões numéricas** — só `int(v)` (inteiro → `int`) e `float(v)`
    (inteiro → `float`).
    `u8(v)`/`i8(v)`/`u16(v)`/`u32(v)`/`u64(v)` não existem, `float32` não
    converte para `float`, e literal inteiro não adapta a `u8` (`42u8`).
14. **Campos de `range`** — `r.start`/`r.end` não são acessíveis em 0.3.8
    (`type.field_on_non_struct`); itere com `for` e guarde os extremos.
15. **`.or_return()` em binding local** — funciona sobre chamada; sobre um
    `const` local reporta `type.field_on_non_struct`.
16. **`simd`** — `u32`/`u64` não são elementos válidos; o checker lista
    `float32/int32`, `float64/int64`, `int16/u16`, `int8/u8`
    (`type.invalid_simd_type`).
17. **Fatias abertas** — `xs[2..]` e `xs[..]` compilam mas abortam no runtime
    (`ori list slice bounds out of range`); use fim explícito (`xs[2..len]`).
18. **Pipe** — `a |> f` aceita só função de um argumento; `a |> f(b)` não é
    suportado em 0.3.8.
18. **`len` em coleção** — o `len` global aceita só `string`; para listas use
    `ori.list.len` / `lists.len`.

---

## O que memorizar

1. **`module` primeiro, imports depois, declarações por último.**
2. **Bloco fecha com `end`**; rótulo (`end if`, `end match`) é opcional.
3. **Sem `func` de declaração**; `func(...) -> T` é só tipo.
4. **Tipos compostos com `[]`**, nunca `<>`.
5. **`optional[T]` para ausência, `result[T,E]` para falha**, `try` para
   propagar; `?` não existe.
6. **`match` exige `:`** e cobre todos os casos (ou `case else:`).
7. **`apply Tipo: Trait`** (ou `use`) — não há herança de classe.
8. **Coleção não tem método**: `lists.push(xs, v)`.
9. **Conversão é explícita**: `string(42)`, `int(n)`, `42u8`.
10. **`using` fecha recurso em toda saída**, em ordem LIFO.

## Para continuar

| Objetivo | Documento |
|---|---|
| Aprender a linguagem | [Tour](../../language/tour.pt-BR.md) |
| Receitas | [Cookbook](../../guides/cookbook.pt-BR.md) |
| Erros, null e void | [Guia](../../guides/errors-null-void.pt-BR.md) |
| Tipos avançados | [Advanced](../../language/advanced.pt-BR.md) |
| Async e concorrência | [Concurrency](../../language/concurrency.pt-BR.md) |
| FFI e ABI C | [Interop](../../language/interop.pt-BR.md) |
| Especificação normativa | [`docs/spec/`](../../spec/README.md) |
| Catálogo de diagnósticos | [`13-error-catalog.md`](../../spec/13-error-catalog.md) |
| Estado do produto | [`docs/product/status.md`](../../product/status.md) |
