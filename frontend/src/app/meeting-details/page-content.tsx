"use client";

import { useEffect } from 'react';
import { motion } from 'framer-motion';
import Analytics from '@/lib/analytics';
import { TranscriptPanel } from '@/components/MeetingDetails/TranscriptPanel';
import { useCopyOperations } from '@/hooks/meeting-details/useCopyOperations';
import { useMeetingOperations } from '@/hooks/meeting-details/useMeetingOperations';

export default function PageContent({
  meeting,
  onRefetchTranscripts,
  segments,
  hasMore,
  isLoadingMore,
  totalCount,
  loadedCount,
  onLoadMore,
}: {
  meeting: any;
  onRefetchTranscripts?: () => Promise<void>;
  segments?: any[];
  hasMore?: boolean;
  isLoadingMore?: boolean;
  totalCount?: number;
  loadedCount?: number;
  onLoadMore?: () => void;
}) {
  console.log('PAGE CONTENT: Initializing transcript view:', {
    meetingId: meeting.id,
    transcriptsCount: meeting.transcripts?.length,
  });

  const copyOperations = useCopyOperations({
    meeting,
    meetingTitle: meeting.title,
  });

  const meetingOperations = useMeetingOperations({
    meeting,
  });

  useEffect(() => {
    Analytics.trackPageView('meeting_details');
  }, []);

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3, ease: 'easeOut' }}
      className="flex flex-col h-screen bg-gray-50"
    >
      <div className="flex flex-1 overflow-hidden">
        <TranscriptPanel
          meetingTitle={meeting.title}
          meetingCreatedAt={meeting.created_at}
          transcripts={meeting.transcripts}
          onCopyTranscript={copyOperations.handleCopyTranscript}
          onOpenMeetingFolder={meetingOperations.handleOpenMeetingFolder}
          isRecording={false}
          disableAutoScroll={true}
          usePagination={true}
          segments={segments}
          hasMore={hasMore}
          isLoadingMore={isLoadingMore}
          totalCount={totalCount}
          loadedCount={loadedCount}
          onLoadMore={onLoadMore}
          meetingId={meeting.id}
          meetingFolderPath={meeting.folder_path}
          onRefetchTranscripts={onRefetchTranscripts}
        />
      </div>
    </motion.div>
  );
}
