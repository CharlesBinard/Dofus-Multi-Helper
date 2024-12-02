import { createTheme, Loader, MantineProvider } from '@mantine/core';
import '@mantine/core/styles.css';
import { RingLoader } from './components/RingLoader';
import './globalStyles.css';
import { Home } from './pages/Home';

const theme = createTheme({
  components: {
    Loader: Loader.extend({
      defaultProps: {
        loaders: { ...Loader.defaultLoaders, ring: RingLoader },
        type: 'ring',
      },
    }),
  },
});

export const App = () => {
  return (
    <MantineProvider forceColorScheme="dark" theme={theme}>
      <Home />
    </MantineProvider>
  );
};
