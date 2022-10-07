import { Button, ButtonGroup, VisuallyHidden } from '@chakra-ui/react';
import { DiscordLogo, GoogleLogo } from './logos';

const OAuthRedirect = async (provider: 'google' | 'discord') => {
  const res = await fetch(`${process.env.NEXT_PUBLIC_BASE_URL}/auth/${provider}`, {
    credentials: 'include',
  });
  const { uri } = await res.json();
  window.location.href = uri;
};

const providers = [
  {
    name: 'Google',
    bg: 'white',
    icon: <GoogleLogo boxSize="5" />,
    onClick: () => OAuthRedirect('google'),
  },
  {
    name: 'Discord',
    bg: '#7289d9',
    icon: <DiscordLogo boxSize="5" />,
    onClick: () => OAuthRedirect('discord'),
  },
];

export const OAuthButtonGroup = () => (
  <ButtonGroup variant="outline" spacing="4" width="full">
    {providers.map(({ name, bg, icon, onClick }) => (
      <Button key={name} onClick={onClick} width="full" backgroundColor={bg}>
        <VisuallyHidden>Sign in with {name}</VisuallyHidden>
        {icon}
      </Button>
    ))}
  </ButtonGroup>
);
