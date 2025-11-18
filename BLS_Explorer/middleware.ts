import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

/**
 * Middleware for protecting routes that require authentication
 */
export function middleware(request: NextRequest) {
  const token = request.cookies.get('e2_token')?.value;
  const { pathname } = request.nextUrl;

  // Define protected routes
  const protectedPaths = [
    '/wallet',
    '/settings',
  ];

  // Check if the current path is protected
  const isProtectedPath = protectedPaths.some((path) => pathname.startsWith(path));

  // If protected path and no token, redirect to login
  if (isProtectedPath && !token) {
    const loginUrl = new URL('/login', request.url);
    loginUrl.searchParams.set('redirect', pathname);
    return NextResponse.redirect(loginUrl);
  }

  // If on auth pages and already has token, redirect to dashboard
  if ((pathname === '/login' || pathname === '/signup') && token) {
    return NextResponse.redirect(new URL('/', request.url));
  }

  return NextResponse.next();
}

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api (API routes)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico (favicon file)
     */
    '/((?!api|_next/static|_next/image|favicon.ico).*)',
  ],
};
