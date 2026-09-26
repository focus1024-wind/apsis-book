<script lang="ts">
  /**
   * Renders a Rust signature with a light syntax colouring.
   *
   * The colouring is a single pass over the token stream and covers the
   * keywords, the primitive types, and the item name. It is deliberately
   * small: the interface documentation is shown as it is written in the
   * source, and the colour is there to separate the parts at a glance.
   */
  interface Props {
    signature: string;
    name?: string;
  }

  let { signature, name = '' }: Props = $props();

  const KEYWORDS = new Set([
    'pub',
    'fn',
    'struct',
    'enum',
    'trait',
    'impl',
    'const',
    'type',
    'where',
    'as',
    'dyn',
    'mut',
    'ref',
    'unsafe',
    'extern',
    'use',
  ]);

  const TYPES = new Set([
    'f64',
    'f32',
    'u8',
    'u16',
    'u32',
    'u64',
    'usize',
    'i8',
    'i16',
    'i32',
    'i64',
    'isize',
    'bool',
    'str',
    'String',
    'Vec',
    'Option',
    'Result',
    'Self',
  ]);

  interface Token {
    text: string;
    kind: 'keyword' | 'type' | 'name' | 'plain';
  }

  const tokens = $derived.by((): Token[] => {
    const parts = signature.split(/(\s+)/);
    return parts.map((part) => {
      const trimmed = part.trim();
      if (trimmed === '') return { text: part, kind: 'plain' as const };
      if (KEYWORDS.has(trimmed)) return { text: part, kind: 'keyword' as const };
      if (name !== '' && trimmed.startsWith(name)) return { text: part, kind: 'name' as const };
      const base = trimmed.replace(/[<>&(),:;\[\]{}'*]/g, '');
      if (TYPES.has(base)) return { text: part, kind: 'type' as const };
      return { text: part, kind: 'plain' as const };
    });
  });
</script>

<code class="signature">{#each tokens as token}{#if token.kind === 'plain'}{token.text}{:else}<span class={token.kind}>{token.text}</span>{/if}{/each}</code>
