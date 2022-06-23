import { LayoutProps } from '@chakra-ui/react';
import React from 'react';
import { Footer } from './footer';
import NavBar from './navbar';

export const Layout: React.FC<LayoutProps & { children: any }> = ({ children }) => {
  return (
    <>
      <NavBar />
      {children}
      <Footer />
    </>
  );
};
