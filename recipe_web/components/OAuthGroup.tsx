import { Button, ButtonGroup, VisuallyHidden } from '@chakra-ui/react';
import { DiscordLogo } from './logos';

const providers = [{ name: 'Discord', icon: <DiscordLogo boxSize="5" /> }];

export const OAuthButtonGroup = () => (
  <ButtonGroup variant="outline" spacing="4" width="full">
    {providers.map(({ name, icon }) => (
      <Button key={name} width="full" backgroundColor={'#7289d9'}>
        <VisuallyHidden>Sign in with {name}</VisuallyHidden>
        {icon}
      </Button>
    ))}
  </ButtonGroup>
);
