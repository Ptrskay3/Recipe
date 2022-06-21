// import { ColorModeScript } from '@chakra-ui/react';
// import NextDocument, { Html, Head, Main, NextScript } from 'next/document';
// import theme from '../common/theme';

// export default class Document extends NextDocument {
//   render() {
//     return (
//       <Html lang="en">
//         <Head />
//         <body>
//           <ColorModeScript initialColorMode={theme.config.initialColorMode} />
//           <Main />
//           <NextScript />
//         </body>
//       </Html>
//     );
//   }
// }

// https://github.com/chakra-ui/chakra-ui/issues/6192#issuecomment-1159979257
import { ColorMode } from '@chakra-ui/color-mode';
import { ColorModeScript } from '@chakra-ui/react';
import Document, { Html, Head, Main, NextScript, DocumentContext } from 'next/document';

import theme from '../common/theme';

type MaybeColorMode = ColorMode | undefined;

function parseCookie(cookie: string, key: string): MaybeColorMode {
  const match = cookie.match(new RegExp(`(^| )${key}=([^;]+)`));
  return match?.[2] as MaybeColorMode;
}

export default class MyDocument extends Document<{ colorMode: string }> {
  static async getInitialProps(ctx: DocumentContext) {
    const initialProps = await Document.getInitialProps(ctx);

    let colorMode: MaybeColorMode = theme.config.initialColorMode;

    if (ctx.req && ctx.req.headers.cookie) {
      colorMode =
        parseCookie(ctx.req.headers.cookie, 'chakra-ui-color-mode') ||
        theme.config.initialColorMode;
    }

    return { ...initialProps, colorMode };
  }

  render() {
    const { colorMode } = this.props;

    return (
      <Html data-theme={colorMode} lang="en" style={{ colorScheme: colorMode }}>
        <Head />
        <body className={`chakra-ui-${colorMode}`}>
          <ColorModeScript initialColorMode={theme.config.initialColorMode} type="cookie" />
          <Main />
          <NextScript />
        </body>
      </Html>
    );
  }
}
