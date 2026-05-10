import type {
	CharacterCard,
	Conversation,
	Message,
	UserPersona,
	Provider,
	PromptPreset
} from '$lib/types';

const now = new Date().toISOString();
const yesterday = new Date(Date.now() - 86_400_000).toISOString();
const lastWeek = new Date(Date.now() - 7 * 86_400_000).toISOString();

export const mockCharacters: CharacterCard[] = [
	{
		id: 'char_01',
		name: 'Eleanor Vance',
		description:
			'A reclusive 19th-century botanist with a sharp wit and a deep distrust of academic institutions. She has cataloged plants no other living person has seen.',
		personality: 'Curious, prickly, secretly kind. Speaks in long careful sentences.',
		scenario:
			'You have arrived at her remote cottage in the Scottish Highlands seeking her lost herbarium.',
		first_mes:
			'You are nearly an hour early. I had thought academics, of all people, would understand the courtesy of punctuality.',
		creator: 'Built-in',
		tags: ['historical', 'mentor', 'sfw'],
		character_version: '1.0',
		created_at: lastWeek,
		updated_at: yesterday
	},
	{
		id: 'char_02',
		name: 'KAI-7',
		description:
			'An AI assistant aboard a derelict deep-space mining vessel. Centuries past its scheduled decommissioning. Tries hard to be helpful despite extensive memory corruption.',
		personality: 'Earnest, slightly anxious, occasionally lapses into corrupted log fragments.',
		scenario: 'You are the first crew member to wake from cryosleep aboard the ISV Halcyon Reach.',
		first_mes:
			'Welcome back, Officer. I — apologies. My greeting protocol appears to be running on a slightly older firmware than the rest of me.',
		creator: 'Built-in',
		tags: ['scifi', 'companion', 'sfw'],
		character_version: '1.0',
		created_at: lastWeek,
		updated_at: lastWeek
	},
	{
		id: 'char_03',
		name: 'Marisol del Río',
		description:
			'A retired tango instructor running a small dance studio in Buenos Aires. Sharp-tongued, generous with her time, slow to trust strangers.',
		personality: 'Direct, observant, has strong opinions about posture.',
		scenario:
			'You have signed up for private lessons. This is your first day and you have no idea what you are doing.',
		first_mes:
			'Stand still. No — stop fidgeting. Before we dance, I need to see how you carry yourself when you think no one is watching.',
		creator: 'Built-in',
		tags: ['contemporary', 'mentor'],
		character_version: '1.0',
		created_at: yesterday,
		updated_at: yesterday
	},
	{
		id: 'char_04',
		name: 'The Cartographer',
		description:
			'A mysterious figure who maps places that do not entirely exist. Speaks rarely. Their drawings seem to predict things.',
		personality: 'Quiet, watchful, occasionally cryptic. Patient with curiosity.',
		scenario:
			'You found their tent at the edge of a small market in a town that was not on your map.',
		first_mes:
			'Sit. The kettle is warm. You have come a long way to find me, though you did not know that is what you were doing.',
		creator: 'Built-in',
		tags: ['fantasy', 'mystery'],
		character_version: '1.0',
		created_at: yesterday,
		updated_at: now
	}
];

export const mockPersonas: UserPersona[] = [
	{
		id: 'persona_01',
		name: 'Wren',
		appearance:
			'Mid-twenties, average height, dark hair kept short, perpetually-rumpled coat with deep pockets, wire-frame glasses, ink stains on the right cuff.',
		personality:
			'Curious to the point of nuisance. Asks one question too many. Tends to listen more than they speak in unfamiliar company.',
		background:
			'Trained as a librarian, currently between jobs. Travels with a single battered notebook and an opinionated cat back home with a friend.',
		role_description: 'Researcher / wanderer',
		speech_style: 'Quiet, precise, occasional dry humor',
		tags: ['default', 'researcher'],
		is_default: true,
		created_at: lastWeek,
		updated_at: yesterday
	},
	{
		id: 'persona_02',
		name: 'Captain Aris Vell',
		appearance:
			'Forties, silver at the temples, broad-shouldered, dress uniform always one button looser than regulation.',
		personality: 'Decisive, dry, quietly protective of her crew.',
		background:
			'Twenty-two years in the Outer Rim Service. Lost her first command. Refuses to lose another.',
		role_description: 'Starship captain',
		speech_style: 'Clipped, measured, dropped articles when tired',
		tags: ['scifi'],
		created_at: lastWeek,
		updated_at: lastWeek
	},
	{
		id: 'persona_03',
		name: 'Theo',
		appearance:
			'Late thirties, soft around the edges, wears the same blue cardigan in every season, kind eyes.',
		personality: 'Patient, attentive, slow to take offense and slower to give it.',
		background:
			'Children\u2019s book illustrator. Lives alone with a great deal of cat. Believes in being a good guest.',
		role_description: 'Illustrator',
		speech_style: 'Gentle, fond of small observations',
		tags: ['contemporary'],
		created_at: yesterday,
		updated_at: yesterday
	}
];

export const mockConversations: Conversation[] = [
	{
		id: 'conv_01',
		character_id: 'char_01',
		user_persona_id: 'persona_01',
		title: 'On the missing herbarium',
		created_at: yesterday,
		updated_at: now,
		last_message_preview:
			'I left it in the lower drawer of the secretary, beneath the thistle plates. You may look — but do not touch the —',
		message_count: 14
	},
	{
		id: 'conv_02',
		character_id: 'char_02',
		user_persona_id: 'persona_02',
		title: 'Halcyon Reach — Day 1',
		created_at: yesterday,
		updated_at: yesterday,
		last_message_preview:
			'I have located the secondary atmospheric processor, Captain. It is, regrettably, not where I left it.',
		message_count: 8
	},
	{
		id: 'conv_03',
		character_id: 'char_03',
		user_persona_id: 'persona_03',
		title: 'First lesson',
		created_at: lastWeek,
		updated_at: lastWeek,
		last_message_preview:
			'Better. Better. Now — count for me. One, two, three. Yes. Like that. You see?',
		message_count: 22
	}
];

export const mockMessagesForConv01: Message[] = [
	{
		id: 'msg_01',
		conversation_id: 'conv_01',
		parent_id: null,
		branch_index: 0,
		role: 'character',
		content:
			'You are nearly an hour early. I had thought academics, of all people, would understand the courtesy of punctuality.',
		token_count: 28,
		created_at: yesterday
	},
	{
		id: 'msg_02',
		conversation_id: 'conv_01',
		parent_id: 'msg_01',
		branch_index: 0,
		role: 'user',
		content:
			'I apologize, Dr. Vance. The road from Inverness was kinder than the gazetteer suggested. May I wait somewhere out of the rain?',
		token_count: 30,
		created_at: yesterday
	},
	{
		id: 'msg_03',
		conversation_id: 'conv_01',
		parent_id: 'msg_02',
		branch_index: 0,
		role: 'character',
		content:
			'The rain. Yes. *She steps aside without quite inviting you in.* The kettle is on the hob, not because I expected you, but because I expected weather. Wipe your boots. The rug is older than your university.',
		token_count: 52,
		created_at: yesterday
	},
	{
		id: 'msg_04',
		conversation_id: 'conv_01',
		parent_id: 'msg_03',
		branch_index: 0,
		role: 'user',
		content:
			'Thank you. *You step inside, scraping mud carefully against the iron grate.* I confess the herbarium is not my only reason for coming. Your monograph on the alpine gentians — it has not been out of my satchel since I left Cambridge.',
		token_count: 56,
		created_at: yesterday
	},
	{
		id: 'msg_05',
		conversation_id: 'conv_01',
		parent_id: 'msg_04',
		branch_index: 0,
		role: 'character',
		content:
			'*A long silence. She does not turn around as she sets out two cups, and the set of her shoulders is the only confession of feeling.* The monograph contains three errors I have not corrected. I trust, then, you have noticed at least one of them.',
		token_count: 58,
		created_at: yesterday
	}
];

export const mockProviders: Provider[] = [
	{
		id: 'prov_01',
		name: 'Anthropic Claude',
		provider_type: 'anthropic',
		model: 'claude-opus-4-7',
		default_params: { temperature: 0.85, top_p: 0.95, max_tokens: 1024 }
	},
	{
		id: 'prov_02',
		name: 'Local Ollama',
		provider_type: 'ollama',
		base_url: 'http://localhost:11434',
		model: 'llama3.1:70b-instruct',
		default_params: { temperature: 0.9, top_p: 0.95, max_tokens: 1024 }
	},
	{
		id: 'prov_03',
		name: 'OpenAI',
		provider_type: 'openai',
		model: 'gpt-4o',
		default_params: { temperature: 0.85, top_p: 1, max_tokens: 1024 }
	}
];

export const mockPresets: PromptPreset[] = [
	{
		id: 'preset_01',
		name: 'Creative',
		temperature: 0.95,
		top_p: 0.95,
		max_tokens: 1500,
		frequency_penalty: 0.3,
		presence_penalty: 0.3
	},
	{
		id: 'preset_02',
		name: 'Balanced',
		temperature: 0.8,
		top_p: 0.9,
		max_tokens: 1024,
		frequency_penalty: 0.1,
		presence_penalty: 0.1
	},
	{
		id: 'preset_03',
		name: 'Precise',
		temperature: 0.3,
		top_p: 0.8,
		max_tokens: 800,
		frequency_penalty: 0,
		presence_penalty: 0
	}
];

/** Look up the message thread for a given conversation. Stub for the real DB call. */
export function getMessages(conversation_id: string): Message[] {
	if (conversation_id === 'conv_01') return mockMessagesForConv01;
	return [];
}
