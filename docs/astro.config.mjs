import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://wsafight.github.io',
  base: '/gpui-rsx',
  integrations: [
    starlight({
      title: 'GPUI-RSX',
      description: 'A compile-time RSX macro for GPUI.',
      favicon: '/favicon.svg',
      customCss: ['./src/styles/custom.css'],
      components: {
        SiteTitle: './src/components/SiteTitle.astro',
        Hero: './src/components/HomeHero.astro',
        Head: './src/components/SiteHead.astro',
        LanguageSelect: './src/components/LanguageSelect.astro',
      },
      expressiveCode: {
        themes: ['github-dark', 'github-light'],
        styleOverrides: {
          borderRadius: '0.75rem',
          codeFontSize: '0.8125rem',
          codeLineHeight: '1.8',
          codePaddingBlock: '1rem',
        },
      },
      defaultLocale: 'root',
      locales: {
        root: {
          label: 'English',
          lang: 'en',
        },
        'zh-cn': {
          label: '简体中文',
          lang: 'zh-CN',
        },
      },
      editLink: {
        baseUrl: 'https://github.com/wsafight/gpui-rsx/edit/main/docs/',
      },
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/wsafight/gpui-rsx',
        },
      ],
      sidebar: [
        {
          label: 'Start',
          translations: {
            'zh-CN': '开始',
          },
          items: [
            {
              label: 'Overview',
              translations: {
                'zh-CN': '概览',
              },
              slug: '',
            },
            {
              label: 'Getting Started',
              translations: {
                'zh-CN': '快速开始',
              },
              slug: 'getting-started',
            },
            {
              label: 'Compatibility',
              translations: {
                'zh-CN': '兼容性',
              },
              slug: 'compatibility',
            },
            { label: 'Playground', translations: { 'zh-CN': '在线体验' }, slug: 'playground' },
          ],
        },
        {
          label: 'Build your UI',
          translations: {
            'zh-CN': '构建界面',
          },
          items: [
            {
              label: 'Syntax Reference',
              translations: {
                'zh-CN': '语法参考',
              },
              slug: 'usage/syntax',
            },
            {
              label: 'Class Handling',
              translations: {
                'zh-CN': 'Class 处理',
              },
              slug: 'usage/class',
            },
            {
              label: 'Themes & Design Tokens',
              translations: { 'zh-CN': '主题与设计令牌' },
              slug: 'guides/theming',
            },
            {
              label: 'IDs and Keys',
              translations: {
                'zh-CN': 'ID 与 Key',
              },
              slug: 'usage/ids',
            },
            {
              label: 'gpui-component',
              slug: 'guides/gpui-component',
            },
          ],
        },
        {
          label: 'Examples',
          translations: {
            'zh-CN': '示例',
          },
          items: [
            {
              label: 'Incident Console',
              translations: {
                'zh-CN': '事故响应控制台',
              },
              slug: 'examples/incident-console',
            },
          ],
        },
        {
          label: 'Workflow',
          translations: {
            'zh-CN': '开发工作流',
          },
          items: [
            { label: 'CLI & Editors', translations: { 'zh-CN': 'CLI 与编辑器' }, slug: 'guides/tooling' },
            {
              label: 'Performance',
              translations: {
                'zh-CN': '性能优化',
              },
              slug: 'guides/performance',
            },
            {
              label: 'Best Practices',
              translations: {
                'zh-CN': '最佳实践',
              },
              slug: 'guides/best-practices',
            },
            {
              label: 'Migration Guide',
              translations: {
                'zh-CN': '迁移指南',
              },
              slug: 'guides/migration',
            },
            {
              label: 'Troubleshooting',
              translations: {
                'zh-CN': '问题排查',
              },
              slug: 'guides/troubleshooting',
            },
          ],
        },
        {
          label: 'Reference',
          translations: {
            'zh-CN': '参考',
          },
          items: [
            {
              label: 'API Reference',
              translations: {
                'zh-CN': 'API 参考',
              },
              slug: 'reference/api',
            },
            {
              label: 'Release Checklist',
              translations: {
                'zh-CN': '发布检查清单',
              },
              slug: 'reference/release-checklist',
            },
          ],
        },
      ],
    }),
  ],
});
