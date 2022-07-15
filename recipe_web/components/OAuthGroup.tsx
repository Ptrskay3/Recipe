import { Button, ButtonGroup, VisuallyHidden } from '@chakra-ui/react';
import { DiscordLogo } from './logos';

const discordOAuth = async () => {
  const res = await fetch('http://localhost:3000/auth/discord');
  const { uri } = await res.json();
  window.location.href = uri;
};

const providers = [{ name: 'Discord', icon: <DiscordLogo boxSize="5" />, onClick: discordOAuth }];

export const OAuthButtonGroup = () => (
  <ButtonGroup variant="outline" spacing="4" width="full">
    {providers.map(({ name, icon, onClick }) => (
      <Button key={name} onClick={onClick} width="full" backgroundColor={'#7289d9'}>
        <VisuallyHidden>Sign in with {name}</VisuallyHidden>
        {icon}
      </Button>
    ))}
  </ButtonGroup>
);
