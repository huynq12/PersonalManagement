import { describe, expect, it } from 'vitest'
import en from './locales/en.json'
import vi from './locales/vi.json'

type Tree = { [key: string]: string | Tree }

function flattenKeys(tree: Tree, prefix = ''): string[] {
  return Object.entries(tree).flatMap(([key, value]) =>
    typeof value === 'string' ? [`${prefix}${key}`] : flattenKeys(value, `${prefix}${key}.`),
  )
}

describe('locale files', () => {
  it('define exactly the same keys in English and Vietnamese', () => {
    expect(flattenKeys(vi).sort()).toEqual(flattenKeys(en).sort())
  })

  it('have no empty translations', () => {
    const empty = [en, vi].flatMap((locale) =>
      flattenKeys(locale).filter((key) => {
        const value = key.split('.').reduce<string | Tree>((node, part) => (node as Tree)[part], locale)
        return typeof value !== 'string' || value.trim() === ''
      }),
    )

    expect(empty).toEqual([])
  })
})
