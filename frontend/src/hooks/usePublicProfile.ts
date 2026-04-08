/* [084A-7] Hook para página pública de perfil de usuario.
 * Carga perfil, reviews recibidas/dadas y distribución de ratings.
 * Tabs para alternar entre reviews recibidas y dadas. */

import {useState} from 'react';
import {useQuery} from '@tanstack/react-query';
import {
    apiGetPublicProfile,
    apiGetReviewsReceived,
    apiGetReviewsGiven,
    apiGetRatingDistribution,
} from '../api/publicUsers';

export type ReviewTab = 'received' | 'given';

export function usePublicProfile(username: string | undefined) {
    const [reviewTab, setReviewTab] = useState<ReviewTab>('received');
    const [receivedPage, setReceivedPage] = useState(1);
    const [givenPage, setGivenPage] = useState(1);

    const profileQuery = useQuery({
        queryKey: ['publicProfile', username],
        queryFn: () => apiGetPublicProfile(username!),
        enabled: !!username,
    });

    const ratingsQuery = useQuery({
        queryKey: ['publicRatings', username],
        queryFn: () => apiGetRatingDistribution(username!),
        enabled: !!username,
    });

    const receivedQuery = useQuery({
        queryKey: ['publicReviewsReceived', username, receivedPage],
        queryFn: () => apiGetReviewsReceived(username!, receivedPage),
        enabled: !!username && reviewTab === 'received',
    });

    const givenQuery = useQuery({
        queryKey: ['publicReviewsGiven', username, givenPage],
        queryFn: () => apiGetReviewsGiven(username!, givenPage),
        enabled: !!username && reviewTab === 'given',
    });

    return {
        profile: profileQuery.data,
        isLoadingProfile: profileQuery.isLoading,
        profileError: profileQuery.error,
        ratings: ratingsQuery.data,
        receivedReviews: receivedQuery.data,
        givenReviews: givenQuery.data,
        isLoadingReviews: reviewTab === 'received' ? receivedQuery.isLoading : givenQuery.isLoading,
        reviewTab,
        setReviewTab,
        receivedPage,
        setReceivedPage,
        givenPage,
        setGivenPage,
    };
}
