/* [084A-7] API client para endpoints públicos de perfil de usuario.
 * Consumidos por la página /usuario/:username sin autenticación. */

import instance from './axios-instance';

export interface PublicUserProfile {
    username: string;
    display_name: string | null;
    avatar_url: string | null;
    bio: string | null;
    specialties: string[] | null;
    average_rating: number | null;
    total_completed_orders: number | null;
    linkedin: string | null;
    twitter: string | null;
    website: string | null;
    member_since: string;
}

export interface PublicReviewItem {
    id: string;
    rating: number;
    comment: string | null;
    employee_response: string | null;
    author_name: string | null;
    author_avatar: string | null;
    author_username: string | null;
    service_title: string | null;
    created_at: string;
}

export interface PaginatedPublicReviews {
    reviews: PublicReviewItem[];
    total: number;
    page: number;
    per_page: number;
}

export interface RatingDistribution {
    stars_5: number;
    stars_4: number;
    stars_3: number;
    stars_2: number;
    stars_1: number;
    total: number;
    average: number;
}

export async function apiGetPublicProfile(username: string): Promise<PublicUserProfile> {
    const {data} = await instance.get<PublicUserProfile>(`/api/users/${username}`);
    return data;
}

export async function apiGetReviewsReceived(
    username: string,
    page = 1,
    perPage = 10
): Promise<PaginatedPublicReviews> {
    const {data} = await instance.get<PaginatedPublicReviews>(
        `/api/users/${username}/reviews/received`,
        {params: {page, per_page: perPage}}
    );
    return data;
}

export async function apiGetReviewsGiven(
    username: string,
    page = 1,
    perPage = 10
): Promise<PaginatedPublicReviews> {
    const {data} = await instance.get<PaginatedPublicReviews>(
        `/api/users/${username}/reviews/given`,
        {params: {page, per_page: perPage}}
    );
    return data;
}

export async function apiGetRatingDistribution(username: string): Promise<RatingDistribution> {
    const {data} = await instance.get<RatingDistribution>(`/api/users/${username}/ratings`);
    return data;
}
